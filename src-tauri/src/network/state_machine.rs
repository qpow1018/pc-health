use super::{
    domain::{
        DiagnosticArea, DiagnosticLifecycle, EvidenceSource, NetworkDiagnosticStatus,
        RuntimeAvailability,
    },
    observation::DiagnosticAssessment,
};

#[cfg_attr(not(test), allow(dead_code))]
pub struct NetworkDiagnosticStateMachine {
    status: NetworkDiagnosticStatus,
    previous_failed_sources: Vec<EvidenceSource>,
    incident_area: Option<DiagnosticArea>,
}

#[cfg_attr(not(test), allow(dead_code))]
impl NetworkDiagnosticStateMachine {
    pub fn new() -> Self {
        Self {
            status: NetworkDiagnosticStatus::starting(),
            previous_failed_sources: vec![],
            incident_area: None,
        }
    }

    pub fn status(&self) -> NetworkDiagnosticStatus {
        self.status.clone()
    }

    pub fn apply(
        &mut self,
        assessment: DiagnosticAssessment,
        observed_at: &str,
        full: bool,
    ) -> NetworkDiagnosticStatus {
        let previous_lifecycle = self.status.lifecycle.clone();
        let previous_area = self.status.suspected_area.clone();
        let is_normal = assessment.is_normal();
        let confirmed = self.confirms(&assessment, previous_area.as_ref(), full);

        let (lifecycle, suspected_area) = match (previous_lifecycle, is_normal) {
            (None, true) | (Some(DiagnosticLifecycle::Normal), true) => {
                self.incident_area = None;
                (DiagnosticLifecycle::Normal, None)
            }
            (None, false) | (Some(DiagnosticLifecycle::Normal), false) => {
                (DiagnosticLifecycle::Suspected, assessment.area.clone())
            }
            (Some(DiagnosticLifecycle::Suspected), true) => {
                self.incident_area = None;
                (DiagnosticLifecycle::Normal, None)
            }
            (Some(DiagnosticLifecycle::Suspected), false) if confirmed => {
                self.incident_area = assessment.area.clone();
                (DiagnosticLifecycle::Incident, assessment.area.clone())
            }
            (Some(DiagnosticLifecycle::Suspected), false) => {
                (DiagnosticLifecycle::Suspected, assessment.area.clone())
            }
            (Some(DiagnosticLifecycle::Incident), true) => {
                let area = self.incident_area.clone();
                if area
                    .as_ref()
                    .is_some_and(|area| assessment.verifies_recovery_for(area))
                {
                    (DiagnosticLifecycle::Recovering, area)
                } else {
                    (DiagnosticLifecycle::Incident, area)
                }
            }
            (Some(DiagnosticLifecycle::Incident), false) => {
                (DiagnosticLifecycle::Incident, self.incident_area.clone())
            }
            (Some(DiagnosticLifecycle::Recovering), true) => {
                (DiagnosticLifecycle::Resolved, self.incident_area.clone())
            }
            (Some(DiagnosticLifecycle::Recovering), false) => {
                (DiagnosticLifecycle::Incident, self.incident_area.clone())
            }
            (Some(DiagnosticLifecycle::Resolved), true) => {
                self.incident_area = None;
                (DiagnosticLifecycle::Normal, None)
            }
            (Some(DiagnosticLifecycle::Resolved), false) => {
                self.incident_area = None;
                (DiagnosticLifecycle::Suspected, assessment.area.clone())
            }
        };

        self.status.availability = RuntimeAvailability::Running;
        self.status.lifecycle = Some(lifecycle);
        self.status.suspected_area = suspected_area;
        self.status.observed_at = Some(observed_at.into());
        if full {
            self.status.last_full_probe_at = Some(observed_at.into());
        }
        self.status.evidence = assessment.evidence;
        self.status.error = None;
        self.previous_failed_sources = assessment.failed_sources;

        self.status()
    }

    fn confirms(
        &self,
        assessment: &DiagnosticAssessment,
        previous_area: Option<&DiagnosticArea>,
        full: bool,
    ) -> bool {
        let Some(current_area) = assessment.area.as_ref() else {
            return false;
        };
        if current_area == &DiagnosticArea::Unknown {
            return false;
        }
        if previous_area == Some(current_area) {
            return true;
        }
        if previous_area != Some(&DiagnosticArea::Unknown) || !full {
            return false;
        }

        let retains_failure = assessment
            .failed_sources
            .iter()
            .any(|source| self.previous_failed_sources.contains(source));
        let adds_independent_failure = assessment
            .failed_sources
            .iter()
            .any(|source| !self.previous_failed_sources.contains(source));
        retains_failure && adds_independent_failure
    }
}

impl Default for NetworkDiagnosticStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::domain::{
        DiagnosticArea, DiagnosticEvidence, DiagnosticLifecycle, EvidenceSource, EvidenceStatus,
        RuntimeAvailability,
    };
    use crate::network::observation::DiagnosticAssessment;

    fn assessment(
        area: Option<DiagnosticArea>,
        failed: Vec<EvidenceSource>,
        checked: Vec<EvidenceSource>,
    ) -> DiagnosticAssessment {
        DiagnosticAssessment {
            area,
            evidence: checked
                .iter()
                .map(|source| DiagnosticEvidence {
                    source: source.clone(),
                    status: if failed.contains(source) {
                        EvidenceStatus::Failure
                    } else {
                        EvidenceStatus::Success
                    },
                    checked_at: Some("evidence-time".into()),
                    duration_ms: Some(1),
                    detail: None,
                })
                .collect(),
            failed_sources: failed,
            checked_sources: checked,
        }
    }

    fn normal(checked: Vec<EvidenceSource>) -> DiagnosticAssessment {
        assessment(None, vec![], checked)
    }

    fn gateway_failure() -> DiagnosticAssessment {
        assessment(
            Some(DiagnosticArea::GatewayOrLocal),
            vec![EvidenceSource::Gateway],
            vec![EvidenceSource::Gateway],
        )
    }

    fn external_failure(failed: Vec<EvidenceSource>) -> DiagnosticAssessment {
        assessment(
            Some(DiagnosticArea::External),
            failed,
            vec![EvidenceSource::HttpMicrosoft, EvidenceSource::HttpGoogle],
        )
    }

    fn unknown_failure(source: EvidenceSource) -> DiagnosticAssessment {
        assessment(
            Some(DiagnosticArea::Unknown),
            vec![source.clone()],
            vec![source],
        )
    }

    fn confirmed_machine(
        area: DiagnosticArea,
        abnormal: DiagnosticAssessment,
    ) -> NetworkDiagnosticStateMachine {
        let mut machine = NetworkDiagnosticStateMachine::new();
        machine.apply(abnormal.clone(), "t0", true);
        let status = machine.apply(abnormal, "t1", false);
        assert_eq!(status.lifecycle, Some(DiagnosticLifecycle::Incident));
        assert_eq!(status.suspected_area, Some(area));
        machine
    }

    #[test]
    fn normal_startup_becomes_normal() {
        let mut machine = NetworkDiagnosticStateMachine::new();

        let status = machine.apply(normal(vec![EvidenceSource::Gateway]), "t0", true);

        assert_eq!(status.availability, RuntimeAvailability::Running);
        assert_eq!(status.lifecycle, Some(DiagnosticLifecycle::Normal));
        assert_eq!(status.last_full_probe_at.as_deref(), Some("t0"));
    }

    #[test]
    fn abnormal_start_requires_confirmation_before_incident() {
        let mut machine = NetworkDiagnosticStateMachine::new();

        let first = machine.apply(gateway_failure(), "t0", true);
        let second = machine.apply(gateway_failure(), "t1", false);

        assert_eq!(first.lifecycle, Some(DiagnosticLifecycle::Suspected));
        assert_eq!(second.lifecycle, Some(DiagnosticLifecycle::Incident));
    }

    #[test]
    fn unknown_only_never_confirms_incident() {
        let mut machine = NetworkDiagnosticStateMachine::new();
        machine.apply(unknown_failure(EvidenceSource::HttpMicrosoft), "t0", false);

        let status = machine.apply(unknown_failure(EvidenceSource::HttpMicrosoft), "t1", true);

        assert_eq!(status.lifecycle, Some(DiagnosticLifecycle::Suspected));
        assert_eq!(status.suspected_area, Some(DiagnosticArea::Unknown));
    }

    #[test]
    fn focused_refinement_confirms_unknown_with_shared_and_independent_failures() {
        let mut machine = NetworkDiagnosticStateMachine::new();
        machine.apply(unknown_failure(EvidenceSource::HttpMicrosoft), "t0", false);

        let status = machine.apply(
            external_failure(vec![
                EvidenceSource::HttpMicrosoft,
                EvidenceSource::HttpGoogle,
            ]),
            "t1",
            true,
        );

        assert_eq!(status.lifecycle, Some(DiagnosticLifecycle::Incident));
        assert_eq!(status.suspected_area, Some(DiagnosticArea::External));
    }

    #[test]
    fn suspected_normal_returns_to_normal() {
        let mut machine = NetworkDiagnosticStateMachine::new();
        machine.apply(gateway_failure(), "t0", false);

        let status = machine.apply(normal(vec![EvidenceSource::Gateway]), "t1", false);

        assert_eq!(status.lifecycle, Some(DiagnosticLifecycle::Normal));
        assert_eq!(status.suspected_area, None);
    }

    #[test]
    fn incident_requires_two_normal_observations_to_resolve_then_normal() {
        let mut machine = confirmed_machine(DiagnosticArea::GatewayOrLocal, gateway_failure());

        let recovering = machine.apply(normal(vec![EvidenceSource::Gateway]), "t2", false);
        let resolved = machine.apply(normal(vec![EvidenceSource::Gateway]), "t3", false);
        let normal = machine.apply(normal(vec![EvidenceSource::Gateway]), "t4", false);

        assert_eq!(recovering.lifecycle, Some(DiagnosticLifecycle::Recovering));
        assert_eq!(
            recovering.suspected_area,
            Some(DiagnosticArea::GatewayOrLocal)
        );
        assert_eq!(resolved.lifecycle, Some(DiagnosticLifecycle::Resolved));
        assert_eq!(normal.lifecycle, Some(DiagnosticLifecycle::Normal));
        assert_eq!(normal.suspected_area, None);
    }

    #[test]
    fn recovery_relapse_returns_to_prior_incident() {
        let mut machine = confirmed_machine(DiagnosticArea::GatewayOrLocal, gateway_failure());
        machine.apply(normal(vec![EvidenceSource::Gateway]), "t2", false);

        let status = machine.apply(gateway_failure(), "t3", false);

        assert_eq!(status.lifecycle, Some(DiagnosticLifecycle::Incident));
        assert_eq!(status.suspected_area, Some(DiagnosticArea::GatewayOrLocal));
    }

    #[test]
    fn resolved_new_suspicion_starts_new_suspected() {
        let mut machine = confirmed_machine(DiagnosticArea::GatewayOrLocal, gateway_failure());
        machine.apply(normal(vec![EvidenceSource::Gateway]), "t2", false);
        machine.apply(normal(vec![EvidenceSource::Gateway]), "t3", false);

        let status = machine.apply(unknown_failure(EvidenceSource::HttpMicrosoft), "t4", false);

        assert_eq!(status.lifecycle, Some(DiagnosticLifecycle::Suspected));
        assert_eq!(status.suspected_area, Some(DiagnosticArea::Unknown));
    }

    #[test]
    fn dns_incident_recovery_requires_both_dns_sources_checked() {
        let dns_failure = assessment(
            Some(DiagnosticArea::Dns),
            vec![EvidenceSource::DnsMicrosoft, EvidenceSource::DnsGoogle],
            vec![EvidenceSource::DnsMicrosoft, EvidenceSource::DnsGoogle],
        );
        let mut machine = confirmed_machine(DiagnosticArea::Dns, dns_failure);

        let insufficient = machine.apply(normal(vec![EvidenceSource::Gateway]), "t2", false);
        let sufficient = machine.apply(
            normal(vec![
                EvidenceSource::DnsMicrosoft,
                EvidenceSource::DnsGoogle,
            ]),
            "t3",
            true,
        );

        assert_eq!(insufficient.lifecycle, Some(DiagnosticLifecycle::Incident));
        assert_eq!(sufficient.lifecycle, Some(DiagnosticLifecycle::Recovering));
    }

    #[test]
    fn latest_evidence_and_observed_at_always_replace_previous_values() {
        let mut machine = NetworkDiagnosticStateMachine::new();
        machine.apply(gateway_failure(), "t0", false);
        let latest = unknown_failure(EvidenceSource::HttpGoogle);

        let status = machine.apply(latest.clone(), "t1", true);

        assert_eq!(status.observed_at.as_deref(), Some("t1"));
        assert_eq!(status.last_full_probe_at.as_deref(), Some("t1"));
        assert_eq!(status.evidence, latest.evidence);
        assert_eq!(machine.status(), status);
    }
}
