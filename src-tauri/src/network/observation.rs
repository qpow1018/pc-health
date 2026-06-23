use super::{
    collector::{GOOGLE_HOSTNAME, GOOGLE_URL, MICROSOFT_HOSTNAME, MICROSOFT_URL},
    domain::{
        DiagnosticArea, DiagnosticEvidence, DnsCheck, EvidenceSource, EvidenceStatus, HttpCheck,
        NetworkProbeSnapshot, ProbeError, ProbeStatus,
    },
};

const ETHERNET_IF_TYPE: u32 = 6;
const MAX_DETAIL_CHARS: usize = 256;

#[derive(Clone, Debug)]
#[cfg_attr(not(test), allow(dead_code))]
pub struct DiagnosticAssessment {
    pub area: Option<DiagnosticArea>,
    pub evidence: Vec<DiagnosticEvidence>,
    pub failed_sources: Vec<EvidenceSource>,
    pub checked_sources: Vec<EvidenceSource>,
}

#[cfg_attr(not(test), allow(dead_code))]
impl DiagnosticAssessment {
    pub fn is_normal(&self) -> bool {
        self.area.is_none()
    }

    pub fn verifies_recovery_for(&self, area: &DiagnosticArea) -> bool {
        let required = match area {
            DiagnosticArea::LocalConnection => [
                EvidenceSource::Ethernet,
                EvidenceSource::Ipv4,
                EvidenceSource::DefaultRoute,
            ]
            .as_slice(),
            DiagnosticArea::GatewayOrLocal => [EvidenceSource::Gateway].as_slice(),
            DiagnosticArea::Dns => {
                [EvidenceSource::DnsMicrosoft, EvidenceSource::DnsGoogle].as_slice()
            }
            DiagnosticArea::External => {
                [EvidenceSource::HttpMicrosoft, EvidenceSource::HttpGoogle].as_slice()
            }
            DiagnosticArea::Unknown => return false,
        };

        self.is_normal()
            && required
                .iter()
                .all(|source| self.checked_sources.contains(source))
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub fn assess(snapshot: NetworkProbeSnapshot) -> DiagnosticAssessment {
    let mut evidence = Vec::new();
    let mut failed_sources = Vec::new();
    let mut checked_sources = Vec::new();
    let collected_at = snapshot.collected_at.as_str();

    let active_ethernet = snapshot
        .adapters
        .iter()
        .any(|adapter| adapter.if_type == ETHERNET_IF_TYPE && adapter.operational_status == "up");
    let active_ipv4 = snapshot.adapters.iter().any(|adapter| {
        adapter.if_type == ETHERNET_IF_TYPE
            && adapter.operational_status == "up"
            && !adapter.ipv4_addresses.is_empty()
    });
    let default_route = snapshot.selected_route.as_ref().is_some_and(|route| {
        route.adapter_is_ethernet && route.adapter_is_up && !route.gateway.is_empty()
    });
    let inventory_failed = !snapshot.errors.is_empty();

    push_topology_evidence(
        &mut evidence,
        &mut failed_sources,
        &mut checked_sources,
        EvidenceSource::Ethernet,
        active_ethernet,
        inventory_failed,
        collected_at,
    );
    push_topology_evidence(
        &mut evidence,
        &mut failed_sources,
        &mut checked_sources,
        EvidenceSource::Ipv4,
        active_ipv4,
        inventory_failed,
        collected_at,
    );
    push_topology_evidence(
        &mut evidence,
        &mut failed_sources,
        &mut checked_sources,
        EvidenceSource::DefaultRoute,
        default_route,
        inventory_failed,
        collected_at,
    );

    push_probe_evidence(
        &mut evidence,
        &mut failed_sources,
        &mut checked_sources,
        EvidenceSource::Gateway,
        &snapshot.gateway_check.status,
        snapshot.gateway_check.duration_ms,
        snapshot.gateway_check.error.as_ref(),
        snapshot.gateway_check.reply_address.as_deref(),
        collected_at,
    );

    for check in &snapshot.dns_checks {
        if let Some(source) = dns_source(check) {
            push_probe_evidence(
                &mut evidence,
                &mut failed_sources,
                &mut checked_sources,
                source,
                &check.status,
                check.duration_ms,
                check.error.as_ref(),
                (!check.addresses.is_empty())
                    .then(|| check.addresses.join(", "))
                    .as_deref(),
                collected_at,
            );
        }
    }

    for check in &snapshot.http_checks {
        if let Some(source) = http_source(check) {
            let result_detail = check.status_code.map(|status| format!("HTTP {status}"));
            push_probe_evidence(
                &mut evidence,
                &mut failed_sources,
                &mut checked_sources,
                source,
                &check.status,
                check.duration_ms,
                check.error.as_ref(),
                result_detail.as_deref(),
                collected_at,
            );
        }
    }

    let topology_missing = !(active_ethernet && active_ipv4 && default_route);
    let gateway_failed = failed_with_status(&snapshot.gateway_check.status);
    let gateway_success = snapshot.gateway_check.status == ProbeStatus::Success;
    let microsoft_dns = status_for_dns(&snapshot.dns_checks, MICROSOFT_HOSTNAME);
    let google_dns = status_for_dns(&snapshot.dns_checks, GOOGLE_HOSTNAME);
    let microsoft_http = status_for_http(&snapshot.http_checks, MICROSOFT_URL);
    let google_http = status_for_http(&snapshot.http_checks, GOOGLE_URL);
    let both_dns_failed = both_match(microsoft_dns, google_dns, failed_with_status);
    let both_dns_succeeded = both_match(microsoft_dns, google_dns, |status| {
        status == &ProbeStatus::Success
    });
    let both_http_failed = both_match(microsoft_http, google_http, failed_with_status);
    let any_dns_failed = [microsoft_dns, google_dns]
        .into_iter()
        .flatten()
        .any(failed_with_status);
    let any_http_failed = [microsoft_http, google_http]
        .into_iter()
        .flatten()
        .any(failed_with_status);
    let any_http_succeeded = [microsoft_http, google_http]
        .into_iter()
        .flatten()
        .any(|status| status == &ProbeStatus::Success);
    let unavailable_result = evidence.iter().any(|item| {
        matches!(item.status, EvidenceStatus::Unavailable)
            && !matches!(
                item.source,
                EvidenceSource::Ethernet | EvidenceSource::Ipv4 | EvidenceSource::DefaultRoute
            )
    });

    let area = if inventory_failed && topology_missing {
        Some(DiagnosticArea::Unknown)
    } else if topology_missing {
        Some(DiagnosticArea::LocalConnection)
    } else if gateway_failed {
        Some(DiagnosticArea::GatewayOrLocal)
    } else if both_dns_failed && any_http_succeeded {
        Some(DiagnosticArea::Unknown)
    } else if gateway_success && both_dns_failed {
        Some(DiagnosticArea::Dns)
    } else if gateway_success && both_dns_succeeded && both_http_failed {
        Some(DiagnosticArea::External)
    } else if any_dns_failed || any_http_failed || unavailable_result {
        Some(DiagnosticArea::Unknown)
    } else {
        None
    };

    DiagnosticAssessment {
        area,
        evidence,
        failed_sources,
        checked_sources,
    }
}

fn push_topology_evidence(
    evidence: &mut Vec<DiagnosticEvidence>,
    failed_sources: &mut Vec<EvidenceSource>,
    checked_sources: &mut Vec<EvidenceSource>,
    source: EvidenceSource,
    present: bool,
    inventory_failed: bool,
    collected_at: &str,
) {
    let confirmed = present || !inventory_failed;
    let status = match (present, inventory_failed) {
        (true, _) => EvidenceStatus::Success,
        (false, true) => EvidenceStatus::Unavailable,
        (false, false) => EvidenceStatus::Failure,
    };
    if confirmed {
        checked_sources.push(source.clone());
    }
    if confirmed && !present {
        failed_sources.push(source.clone());
    }
    evidence.push(DiagnosticEvidence {
        source,
        status,
        checked_at: confirmed.then(|| collected_at.into()),
        duration_ms: None,
        detail: None,
    });
}

#[allow(clippy::too_many_arguments)]
fn push_probe_evidence(
    evidence: &mut Vec<DiagnosticEvidence>,
    failed_sources: &mut Vec<EvidenceSource>,
    checked_sources: &mut Vec<EvidenceSource>,
    source: EvidenceSource,
    status: &ProbeStatus,
    duration_ms: u64,
    error: Option<&ProbeError>,
    result_detail: Option<&str>,
    collected_at: &str,
) {
    let actually_checked = matches!(
        status,
        ProbeStatus::Success | ProbeStatus::Timeout | ProbeStatus::Error
    );
    let evidence_status = match status {
        ProbeStatus::Success => EvidenceStatus::Success,
        ProbeStatus::Timeout => EvidenceStatus::Timeout,
        ProbeStatus::Error => EvidenceStatus::Failure,
        ProbeStatus::Unsupported => EvidenceStatus::Unavailable,
        ProbeStatus::NotRun => EvidenceStatus::NotChecked,
    };
    if actually_checked {
        checked_sources.push(source.clone());
    }
    if failed_with_status(status) {
        failed_sources.push(source.clone());
    }
    evidence.push(DiagnosticEvidence {
        source,
        status: evidence_status,
        checked_at: actually_checked.then(|| collected_at.into()),
        duration_ms: actually_checked.then_some(duration_ms),
        detail: bounded_detail(error.map(|error| error.message.as_str()).or(result_detail)),
    });
}

fn bounded_detail(detail: Option<&str>) -> Option<String> {
    detail.map(|detail| detail.chars().take(MAX_DETAIL_CHARS).collect())
}

fn failed_with_status(status: &ProbeStatus) -> bool {
    matches!(status, ProbeStatus::Timeout | ProbeStatus::Error)
}

fn status_for_dns<'a>(checks: &'a [DnsCheck], hostname: &str) -> Option<&'a ProbeStatus> {
    checks
        .iter()
        .find(|check| check.hostname == hostname)
        .map(|check| &check.status)
}

fn status_for_http<'a>(checks: &'a [HttpCheck], url: &str) -> Option<&'a ProbeStatus> {
    checks
        .iter()
        .find(|check| check.url == url)
        .map(|check| &check.status)
}

fn both_match(
    first: Option<&ProbeStatus>,
    second: Option<&ProbeStatus>,
    predicate: impl Fn(&ProbeStatus) -> bool,
) -> bool {
    first.is_some_and(&predicate) && second.is_some_and(predicate)
}

fn dns_source(check: &DnsCheck) -> Option<EvidenceSource> {
    match check.hostname.as_str() {
        MICROSOFT_HOSTNAME => Some(EvidenceSource::DnsMicrosoft),
        GOOGLE_HOSTNAME => Some(EvidenceSource::DnsGoogle),
        _ => None,
    }
}

fn http_source(check: &HttpCheck) -> Option<EvidenceSource> {
    match check.url.as_str() {
        MICROSOFT_URL => Some(EvidenceSource::HttpMicrosoft),
        GOOGLE_URL => Some(EvidenceSource::HttpGoogle),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::{
        collector::{GOOGLE_HOSTNAME, GOOGLE_URL, MICROSOFT_HOSTNAME, MICROSOFT_URL},
        domain::{
            AdapterSnapshot, DiagnosticArea, EvidenceSource, EvidenceStatus, GatewayCheck,
            HttpCheck, NetworkProbeSnapshot, ProbeError, ProbeStatus, RouteSnapshot,
        },
    };

    fn route() -> RouteSnapshot {
        RouteSnapshot {
            interface_index: 7,
            gateway: "192.168.0.1".into(),
            route_metric: 10,
            interface_metric: 5,
            combined_metric: 15,
            adapter_is_ethernet: true,
            adapter_is_up: true,
        }
    }

    fn snapshot() -> NetworkProbeSnapshot {
        let route = route();
        NetworkProbeSnapshot {
            collected_at: "2026-06-23T00:00:00Z".into(),
            collector: "fake".into(),
            duration_ms: 10,
            adapters: vec![AdapterSnapshot {
                name: "ethernet".into(),
                friendly_name: "Ethernet".into(),
                interface_index: 7,
                if_type: 6,
                operational_status: "up".into(),
                mac_address: None,
                ipv4_addresses: vec!["192.168.0.2".into()],
            }],
            default_routes: vec![route.clone()],
            selected_route: Some(route),
            gateway_check: GatewayCheck {
                status: ProbeStatus::Success,
                duration_ms: 2,
                reply_address: Some("192.168.0.1".into()),
                round_trip_ms: Some(1),
                error: None,
            },
            dns_checks: vec![],
            http_checks: vec![],
            errors: vec![],
        }
    }

    fn error(stage: &str) -> ProbeError {
        ProbeError {
            stage: stage.into(),
            code: "probe_failed".into(),
            message: "bounded failure detail".into(),
            native_code: None,
        }
    }

    fn dns(hostname: &str, status: ProbeStatus) -> crate::network::domain::DnsCheck {
        crate::network::domain::DnsCheck {
            hostname: hostname.into(),
            status: status.clone(),
            duration_ms: 5,
            addresses: if status == ProbeStatus::Success {
                vec!["203.0.113.1".into()]
            } else {
                vec![]
            },
            error: (status != ProbeStatus::Success).then(|| error("dns")),
        }
    }

    fn http(url: &str, status: ProbeStatus) -> HttpCheck {
        HttpCheck {
            url: url.into(),
            status: status.clone(),
            duration_ms: 7,
            status_code: (status == ProbeStatus::Success).then_some(204),
            body_matches: (status == ProbeStatus::Success).then_some(true),
            error: (status != ProbeStatus::Success).then(|| error("http")),
        }
    }

    fn with_full_checks(
        mut snapshot: NetworkProbeSnapshot,
        dns_statuses: [ProbeStatus; 2],
        http_statuses: [ProbeStatus; 2],
    ) -> NetworkProbeSnapshot {
        snapshot.dns_checks = vec![
            dns(MICROSOFT_HOSTNAME, dns_statuses[0].clone()),
            dns(GOOGLE_HOSTNAME, dns_statuses[1].clone()),
        ];
        snapshot.http_checks = vec![
            http(MICROSOFT_URL, http_statuses[0].clone()),
            http(GOOGLE_URL, http_statuses[1].clone()),
        ];
        snapshot
    }

    #[test]
    fn inventory_error_without_route_is_unknown_not_local_connection() {
        let mut snapshot = snapshot();
        snapshot.adapters.clear();
        snapshot.default_routes.clear();
        snapshot.selected_route = None;
        snapshot.errors.push(error("inventory"));

        let assessment = assess(snapshot);

        assert_eq!(assessment.area, Some(DiagnosticArea::Unknown));
        assert!(assessment.failed_sources.is_empty());
        assert!([
            EvidenceSource::Ethernet,
            EvidenceSource::Ipv4,
            EvidenceSource::DefaultRoute
        ]
        .iter()
        .all(|source| !assessment.checked_sources.contains(source)));
        assert!(assessment.evidence.iter().take(3).all(|evidence| {
            evidence.status == EvidenceStatus::Unavailable && evidence.checked_at.is_none()
        }));
    }

    #[test]
    fn valid_inventory_without_wired_topology_is_local_connection() {
        let mut snapshot = snapshot();
        snapshot.adapters.clear();
        snapshot.default_routes.clear();
        snapshot.selected_route = None;

        let assessment = assess(snapshot);

        assert_eq!(assessment.area, Some(DiagnosticArea::LocalConnection));
        assert_eq!(
            assessment.failed_sources,
            vec![
                EvidenceSource::Ethernet,
                EvidenceSource::Ipv4,
                EvidenceSource::DefaultRoute
            ]
        );
    }

    #[test]
    fn gateway_timeout_identifies_gateway_or_local_and_preserves_error() {
        let mut snapshot = snapshot();
        snapshot.gateway_check.status = ProbeStatus::Timeout;
        snapshot.gateway_check.duration_ms = 1500;
        snapshot.gateway_check.error = Some(error("gateway"));

        let assessment = assess(snapshot);

        assert_eq!(assessment.area, Some(DiagnosticArea::GatewayOrLocal));
        assert!(assessment.failed_sources.contains(&EvidenceSource::Gateway));
        let evidence = assessment
            .evidence
            .iter()
            .find(|evidence| evidence.source == EvidenceSource::Gateway)
            .unwrap();
        assert_eq!(evidence.status, EvidenceStatus::Timeout);
        assert_eq!(evidence.checked_at.as_deref(), Some("2026-06-23T00:00:00Z"));
        assert_eq!(evidence.duration_ms, Some(1500));
        assert_eq!(evidence.detail.as_deref(), Some("bounded failure detail"));
    }

    #[test]
    fn dns_failure_with_successful_http_is_conflicting_unknown() {
        let snapshot = with_full_checks(
            snapshot(),
            [ProbeStatus::Error, ProbeStatus::Error],
            [ProbeStatus::Success, ProbeStatus::Success],
        );

        let assessment = assess(snapshot);

        assert_eq!(assessment.area, Some(DiagnosticArea::Unknown));
    }

    #[test]
    fn gateway_success_and_dns_failures_without_http_success_are_dns() {
        let snapshot = with_full_checks(
            snapshot(),
            [ProbeStatus::Error, ProbeStatus::Timeout],
            [ProbeStatus::NotRun, ProbeStatus::NotRun],
        );

        let assessment = assess(snapshot);

        assert_eq!(assessment.area, Some(DiagnosticArea::Dns));
    }

    #[test]
    fn two_http_failures_after_gateway_and_dns_success_are_external() {
        let snapshot = with_full_checks(
            snapshot(),
            [ProbeStatus::Success, ProbeStatus::Success],
            [ProbeStatus::Timeout, ProbeStatus::Error],
        );

        let assessment = assess(snapshot);

        assert_eq!(assessment.area, Some(DiagnosticArea::External));
    }

    #[test]
    fn one_http_failure_is_unknown() {
        let snapshot = with_full_checks(
            snapshot(),
            [ProbeStatus::Success, ProbeStatus::Success],
            [ProbeStatus::Timeout, ProbeStatus::Success],
        );

        let assessment = assess(snapshot);

        assert_eq!(assessment.area, Some(DiagnosticArea::Unknown));
    }

    #[test]
    fn duplicate_endpoint_results_do_not_replace_independent_sources() {
        let mut snapshot = with_full_checks(
            snapshot(),
            [ProbeStatus::Success, ProbeStatus::Success],
            [ProbeStatus::Timeout, ProbeStatus::Timeout],
        );
        snapshot.http_checks[1].url = MICROSOFT_URL.into();

        let assessment = assess(snapshot);

        assert_eq!(assessment.area, Some(DiagnosticArea::Unknown));
    }

    #[test]
    fn all_checked_results_success_is_normal_with_exact_sources() {
        let snapshot = with_full_checks(
            snapshot(),
            [ProbeStatus::Success, ProbeStatus::Success],
            [ProbeStatus::Success, ProbeStatus::Success],
        );

        let assessment = assess(snapshot);

        assert!(assessment.is_normal());
        assert_eq!(
            assessment.checked_sources,
            vec![
                EvidenceSource::Ethernet,
                EvidenceSource::Ipv4,
                EvidenceSource::DefaultRoute,
                EvidenceSource::Gateway,
                EvidenceSource::DnsMicrosoft,
                EvidenceSource::DnsGoogle,
                EvidenceSource::HttpMicrosoft,
                EvidenceSource::HttpGoogle,
            ]
        );
        assert!(assessment
            .evidence
            .iter()
            .all(|evidence| evidence.checked_at.is_some()));
    }

    #[test]
    fn absent_dns_and_http_are_not_checked_or_emitted() {
        let assessment = assess(snapshot());

        assert_eq!(assessment.evidence.len(), 4);
        assert!(!assessment
            .checked_sources
            .contains(&EvidenceSource::DnsMicrosoft));
        assert!(!assessment
            .checked_sources
            .contains(&EvidenceSource::HttpMicrosoft));
    }

    #[test]
    fn maps_every_probe_status_and_only_tracks_actual_checks() {
        let cases = [
            (ProbeStatus::Success, EvidenceStatus::Success, true, false),
            (ProbeStatus::Timeout, EvidenceStatus::Timeout, true, true),
            (ProbeStatus::Error, EvidenceStatus::Failure, true, true),
            (
                ProbeStatus::Unsupported,
                EvidenceStatus::Unavailable,
                false,
                false,
            ),
            (
                ProbeStatus::NotRun,
                EvidenceStatus::NotChecked,
                false,
                false,
            ),
        ];

        for (probe_status, evidence_status, actually_checked, failed) in cases {
            let mut snapshot = snapshot();
            snapshot.gateway_check.status = probe_status;
            snapshot.gateway_check.error = None;
            let assessment = assess(snapshot);
            let gateway = assessment
                .evidence
                .iter()
                .find(|evidence| evidence.source == EvidenceSource::Gateway)
                .unwrap();

            assert_eq!(gateway.status, evidence_status);
            assert_eq!(gateway.checked_at.is_some(), actually_checked);
            assert_eq!(
                assessment
                    .checked_sources
                    .contains(&EvidenceSource::Gateway),
                actually_checked
            );
            assert_eq!(
                assessment.failed_sources.contains(&EvidenceSource::Gateway),
                failed
            );
        }
    }

    #[test]
    fn unavailable_source_cannot_confirm_a_later_concrete_incident() {
        use crate::network::{
            domain::DiagnosticLifecycle, state_machine::NetworkDiagnosticStateMachine,
        };

        let mut unavailable_snapshot = snapshot();
        unavailable_snapshot.http_checks = vec![http(MICROSOFT_URL, ProbeStatus::Unsupported)];
        let unavailable = assess(unavailable_snapshot);
        let concrete = assess(with_full_checks(
            snapshot(),
            [ProbeStatus::Success, ProbeStatus::Success],
            [ProbeStatus::Error, ProbeStatus::Error],
        ));
        let mut machine = NetworkDiagnosticStateMachine::new();

        let first = machine.apply(unavailable, "t0", false);
        let second = machine.apply(concrete, "t1", true);

        assert_eq!(first.lifecycle, Some(DiagnosticLifecycle::Suspected));
        assert_eq!(second.lifecycle, Some(DiagnosticLifecycle::Suspected));
    }
}
