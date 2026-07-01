use super::{
    domain::{DiagnosticArea, DiagnosticLifecycle, NetworkDiagnosticStatus, NetworkIncidentStatus},
    incident_store::NetworkIncidentStore,
};

#[cfg_attr(not(test), allow(dead_code))]
pub struct NetworkIncidentRecorder {
    store: NetworkIncidentStore,
    open_incident_id: Option<i64>,
    previous: Option<NetworkDiagnosticStatus>,
}

#[cfg_attr(not(test), allow(dead_code))]
impl NetworkIncidentRecorder {
    pub fn new(store: NetworkIncidentStore) -> Self {
        Self {
            store,
            open_incident_id: None,
            previous: None,
        }
    }

    pub fn record(&mut self, status: &NetworkDiagnosticStatus) -> Result<(), String> {
        let Some(observed_at) = status.observed_at.as_deref() else {
            self.previous = Some(status.clone());
            return Ok(());
        };

        self.store.record_probe_observation(
            observed_at,
            status.lifecycle.as_ref().map(DiagnosticLifecycle::as_str),
            status.suspected_area.as_ref().map(DiagnosticArea::as_str),
            &status.evidence,
        )?;

        match status.lifecycle {
            Some(DiagnosticLifecycle::Incident) => self.record_incident(status, observed_at)?,
            Some(DiagnosticLifecycle::Recovering) => {
                self.update_open(status, NetworkIncidentStatus::Recovering, observed_at, None)?
            }
            Some(DiagnosticLifecycle::Resolved) => {
                self.update_open(
                    status,
                    NetworkIncidentStatus::Resolved,
                    observed_at,
                    Some(observed_at),
                )?;
                self.open_incident_id = None;
            }
            Some(DiagnosticLifecycle::Normal | DiagnosticLifecycle::Suspected) | None => {}
        }

        self.previous = Some(status.clone());
        Ok(())
    }

    fn record_incident(
        &mut self,
        status: &NetworkDiagnosticStatus,
        observed_at: &str,
    ) -> Result<(), String> {
        let area = status
            .suspected_area
            .clone()
            .unwrap_or(DiagnosticArea::Unknown);
        let summary = summary_for(status, NetworkIncidentStatus::Ongoing);
        if let Some(id) = self.open_incident_id {
            return self.store.update_incident(
                id,
                NetworkIncidentStatus::Ongoing,
                area,
                observed_at,
                None,
                &summary,
                &status.evidence,
            );
        }

        let id = self
            .store
            .create_incident(area, observed_at, &summary, &status.evidence)?;
        self.open_incident_id = Some(id);
        Ok(())
    }

    fn update_open(
        &mut self,
        status: &NetworkDiagnosticStatus,
        incident_status: NetworkIncidentStatus,
        observed_at: &str,
        resolved_at: Option<&str>,
    ) -> Result<(), String> {
        let Some(id) = self.open_incident_id else {
            return Ok(());
        };
        let area = status
            .suspected_area
            .clone()
            .or_else(|| {
                self.previous
                    .as_ref()
                    .and_then(|previous| previous.suspected_area.clone())
            })
            .unwrap_or(DiagnosticArea::Unknown);
        let summary = summary_for(status, incident_status.clone());
        self.store.update_incident(
            id,
            incident_status,
            area,
            observed_at,
            resolved_at,
            &summary,
            &status.evidence,
        )
    }
}

fn summary_for(status: &NetworkDiagnosticStatus, incident_status: NetworkIncidentStatus) -> String {
    let area = status
        .suspected_area
        .clone()
        .unwrap_or(DiagnosticArea::Unknown);
    let area_label = match area {
        DiagnosticArea::LocalConnection => "내 PC 연결",
        DiagnosticArea::GatewayOrLocal => "내 PC 또는 공유기",
        DiagnosticArea::Dns => "DNS",
        DiagnosticArea::External => "외부 연결",
        DiagnosticArea::Unknown => "확인 불가",
    };

    match incident_status {
        NetworkIncidentStatus::Ongoing => {
            format!("{area_label}에서 문제 근거가 확인되었습니다.")
        }
        NetworkIncidentStatus::Recovering => {
            format!("{area_label}이 정상으로 돌아왔는지 확인하고 있습니다.")
        }
        NetworkIncidentStatus::Resolved => "네트워크 장애가 복구되었습니다.".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::NetworkIncidentRecorder;
    use crate::network::{
        domain::{
            DiagnosticArea, DiagnosticEvidence, DiagnosticLifecycle, EvidenceSource,
            EvidenceStatus, NetworkDiagnosticStatus, NetworkIncidentStatus, RuntimeAvailability,
        },
        incident_store::NetworkIncidentStore,
    };

    fn evidence(status: EvidenceStatus) -> DiagnosticEvidence {
        DiagnosticEvidence {
            source: EvidenceSource::Gateway,
            status,
            checked_at: Some("2026-06-30T00:00:00Z".into()),
            duration_ms: Some(10),
            detail: Some("test evidence".into()),
        }
    }

    fn status(
        lifecycle: DiagnosticLifecycle,
        area: Option<DiagnosticArea>,
        observed_at: &str,
        evidence_status: EvidenceStatus,
    ) -> NetworkDiagnosticStatus {
        NetworkDiagnosticStatus {
            availability: RuntimeAvailability::Running,
            lifecycle: Some(lifecycle),
            suspected_area: area,
            observed_at: Some(observed_at.into()),
            last_full_probe_at: Some(observed_at.into()),
            evidence: vec![evidence(evidence_status)],
            error: None,
        }
    }

    #[test]
    fn suspected_does_not_create_incident_until_confirmed() {
        let store = NetworkIncidentStore::open_in_memory().unwrap();
        let mut recorder = NetworkIncidentRecorder::new(store.clone());

        recorder
            .record(&status(
                DiagnosticLifecycle::Suspected,
                Some(DiagnosticArea::GatewayOrLocal),
                "2026-06-30T00:00:00Z",
                EvidenceStatus::Timeout,
            ))
            .unwrap();

        assert_eq!(store.recent_incidents(3).unwrap().len(), 0);
        assert_eq!(store.probe_observation_count().unwrap(), 1);

        recorder
            .record(&status(
                DiagnosticLifecycle::Incident,
                Some(DiagnosticArea::GatewayOrLocal),
                "2026-06-30T00:00:20Z",
                EvidenceStatus::Timeout,
            ))
            .unwrap();

        let incidents = store.recent_incidents(3).unwrap();
        assert_eq!(incidents.len(), 1);
        assert_eq!(incidents[0].status, NetworkIncidentStatus::Ongoing);
        assert_eq!(incidents[0].started_at, "2026-06-30T00:00:20Z");
        assert_eq!(store.probe_observation_count().unwrap(), 2);
    }

    #[test]
    fn incident_recovery_and_resolved_update_same_record() {
        let store = NetworkIncidentStore::open_in_memory().unwrap();
        let mut recorder = NetworkIncidentRecorder::new(store.clone());

        for (lifecycle, at, evidence_status) in [
            (
                DiagnosticLifecycle::Incident,
                "2026-06-30T00:00:00Z",
                EvidenceStatus::Timeout,
            ),
            (
                DiagnosticLifecycle::Incident,
                "2026-06-30T00:00:20Z",
                EvidenceStatus::Timeout,
            ),
            (
                DiagnosticLifecycle::Recovering,
                "2026-06-30T00:00:40Z",
                EvidenceStatus::Success,
            ),
            (
                DiagnosticLifecycle::Incident,
                "2026-06-30T00:00:50Z",
                EvidenceStatus::Timeout,
            ),
            (
                DiagnosticLifecycle::Recovering,
                "2026-06-30T00:01:00Z",
                EvidenceStatus::Success,
            ),
            (
                DiagnosticLifecycle::Resolved,
                "2026-06-30T00:01:20Z",
                EvidenceStatus::Success,
            ),
        ] {
            recorder
                .record(&status(
                    lifecycle,
                    Some(DiagnosticArea::GatewayOrLocal),
                    at,
                    evidence_status,
                ))
                .unwrap();
        }

        let incidents = store.recent_incidents(3).unwrap();
        assert_eq!(incidents.len(), 1);
        assert_eq!(incidents[0].status, NetworkIncidentStatus::Resolved);
        assert_eq!(incidents[0].started_at, "2026-06-30T00:00:00Z");
        assert_eq!(incidents[0].last_observed_at, "2026-06-30T00:01:20Z");
        assert_eq!(
            incidents[0].resolved_at.as_deref(),
            Some("2026-06-30T00:01:20Z")
        );
    }

    #[test]
    fn resolved_then_new_incident_creates_new_record() {
        let store = NetworkIncidentStore::open_in_memory().unwrap();
        let mut recorder = NetworkIncidentRecorder::new(store.clone());

        for (lifecycle, at, evidence_status) in [
            (
                DiagnosticLifecycle::Incident,
                "2026-06-30T00:00:00Z",
                EvidenceStatus::Timeout,
            ),
            (
                DiagnosticLifecycle::Resolved,
                "2026-06-30T00:00:20Z",
                EvidenceStatus::Success,
            ),
            (
                DiagnosticLifecycle::Normal,
                "2026-06-30T00:00:30Z",
                EvidenceStatus::Success,
            ),
            (
                DiagnosticLifecycle::Suspected,
                "2026-06-30T00:00:40Z",
                EvidenceStatus::Timeout,
            ),
            (
                DiagnosticLifecycle::Incident,
                "2026-06-30T00:01:00Z",
                EvidenceStatus::Timeout,
            ),
        ] {
            recorder
                .record(&status(
                    lifecycle,
                    Some(DiagnosticArea::GatewayOrLocal),
                    at,
                    evidence_status,
                ))
                .unwrap();
        }

        let incidents = store.recent_incidents(3).unwrap();
        assert_eq!(incidents.len(), 2);
        assert_eq!(incidents[0].started_at, "2026-06-30T00:01:00Z");
        assert_eq!(incidents[1].started_at, "2026-06-30T00:00:00Z");
    }
}
