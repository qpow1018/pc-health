use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub enum ProbeStatus {
    Success,
    Timeout,
    NotRun,
    Unsupported,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeError {
    pub stage: String,
    pub code: String,
    pub message: String,
    pub native_code: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeAvailability {
    Starting,
    Running,
    Unavailable,
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticLifecycle {
    Normal,
    Suspected,
    Incident,
    Recovering,
    Resolved,
}

impl DiagnosticLifecycle {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Suspected => "suspected",
            Self::Incident => "incident",
            Self::Recovering => "recovering",
            Self::Resolved => "resolved",
        }
    }

    #[allow(dead_code)]
    pub fn from_str(value: &str) -> Result<Self, String> {
        match value {
            "normal" => Ok(Self::Normal),
            "suspected" => Ok(Self::Suspected),
            "incident" => Ok(Self::Incident),
            "recovering" => Ok(Self::Recovering),
            "resolved" => Ok(Self::Resolved),
            other => Err(format!("unknown diagnostic lifecycle: {other}")),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticArea {
    LocalConnection,
    GatewayOrLocal,
    Dns,
    External,
    Unknown,
}

impl DiagnosticArea {
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LocalConnection => "local_connection",
            Self::GatewayOrLocal => "gateway_or_local",
            Self::Dns => "dns",
            Self::External => "external",
            Self::Unknown => "unknown",
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn from_str(value: &str) -> Result<Self, String> {
        match value {
            "local_connection" => Ok(Self::LocalConnection),
            "gateway_or_local" => Ok(Self::GatewayOrLocal),
            "dns" => Ok(Self::Dns),
            "external" => Ok(Self::External),
            "unknown" => Ok(Self::Unknown),
            other => Err(format!("unknown diagnostic area: {other}")),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSource {
    Ethernet,
    Ipv4,
    DefaultRoute,
    Gateway,
    DnsMicrosoft,
    DnsGoogle,
    HttpMicrosoft,
    HttpGoogle,
}

impl EvidenceSource {
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ethernet => "ethernet",
            Self::Ipv4 => "ipv4",
            Self::DefaultRoute => "default_route",
            Self::Gateway => "gateway",
            Self::DnsMicrosoft => "dns_microsoft",
            Self::DnsGoogle => "dns_google",
            Self::HttpMicrosoft => "http_microsoft",
            Self::HttpGoogle => "http_google",
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn from_str(value: &str) -> Result<Self, String> {
        match value {
            "ethernet" => Ok(Self::Ethernet),
            "ipv4" => Ok(Self::Ipv4),
            "default_route" => Ok(Self::DefaultRoute),
            "gateway" => Ok(Self::Gateway),
            "dns_microsoft" => Ok(Self::DnsMicrosoft),
            "dns_google" => Ok(Self::DnsGoogle),
            "http_microsoft" => Ok(Self::HttpMicrosoft),
            "http_google" => Ok(Self::HttpGoogle),
            other => Err(format!("unknown evidence source: {other}")),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceStatus {
    Success,
    Failure,
    Timeout,
    Unavailable,
    NotChecked,
}

impl EvidenceStatus {
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Failure => "failure",
            Self::Timeout => "timeout",
            Self::Unavailable => "unavailable",
            Self::NotChecked => "not_checked",
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn from_str(value: &str) -> Result<Self, String> {
        match value {
            "success" => Ok(Self::Success),
            "failure" => Ok(Self::Failure),
            "timeout" => Ok(Self::Timeout),
            "unavailable" => Ok(Self::Unavailable),
            "not_checked" => Ok(Self::NotChecked),
            other => Err(format!("unknown evidence status: {other}")),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticEvidence {
    pub source: EvidenceSource,
    pub status: EvidenceStatus,
    pub checked_at: Option<String>,
    pub duration_ms: Option<u64>,
    pub detail: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkDiagnosticStatus {
    pub availability: RuntimeAvailability,
    pub lifecycle: Option<DiagnosticLifecycle>,
    pub suspected_area: Option<DiagnosticArea>,
    pub observed_at: Option<String>,
    pub last_full_probe_at: Option<String>,
    pub evidence: Vec<DiagnosticEvidence>,
    pub error: Option<ProbeError>,
}

impl NetworkDiagnosticStatus {
    pub fn starting() -> Self {
        Self {
            availability: RuntimeAvailability::Starting,
            lifecycle: None,
            suspected_area: None,
            observed_at: None,
            last_full_probe_at: None,
            evidence: vec![],
            error: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(not(test), allow(dead_code))]
pub enum NetworkIncidentStatus {
    Ongoing,
    Recovering,
    Resolved,
}

impl NetworkIncidentStatus {
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ongoing => "ongoing",
            Self::Recovering => "recovering",
            Self::Resolved => "resolved",
        }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn from_str(value: &str) -> Result<Self, String> {
        match value {
            "ongoing" => Ok(Self::Ongoing),
            "recovering" => Ok(Self::Recovering),
            "resolved" => Ok(Self::Resolved),
            other => Err(format!("unknown incident status: {other}")),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(not(test), allow(dead_code))]
pub struct NetworkIncidentEvidence {
    pub source: EvidenceSource,
    pub status: EvidenceStatus,
    pub checked_at: Option<String>,
    pub duration_ms: Option<u64>,
    pub detail: Option<String>,
    pub observed_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(not(test), allow(dead_code))]
pub struct NetworkIncident {
    pub id: i64,
    pub status: NetworkIncidentStatus,
    pub area: DiagnosticArea,
    pub started_at: String,
    pub last_observed_at: String,
    pub resolved_at: Option<String>,
    pub summary: String,
    pub representative_evidence: Vec<NetworkIncidentEvidence>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdapterSnapshot {
    pub name: String,
    pub friendly_name: String,
    pub interface_index: u32,
    pub if_type: u32,
    pub operational_status: String,
    pub mac_address: Option<String>,
    pub ipv4_addresses: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteSnapshot {
    pub interface_index: u32,
    pub gateway: String,
    pub route_metric: u32,
    pub interface_metric: u32,
    pub combined_metric: u32,
    pub adapter_is_ethernet: bool,
    pub adapter_is_up: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GatewayCheck {
    pub status: ProbeStatus,
    pub duration_ms: u64,
    pub reply_address: Option<String>,
    pub round_trip_ms: Option<u32>,
    pub error: Option<ProbeError>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DnsCheck {
    pub hostname: String,
    pub status: ProbeStatus,
    pub duration_ms: u64,
    pub addresses: Vec<String>,
    pub error: Option<ProbeError>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpCheck {
    pub url: String,
    pub status: ProbeStatus,
    pub duration_ms: u64,
    pub status_code: Option<u16>,
    pub body_matches: Option<bool>,
    pub error: Option<ProbeError>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkProbeSnapshot {
    pub collected_at: String,
    pub collector: String,
    pub duration_ms: u64,
    pub adapters: Vec<AdapterSnapshot>,
    pub default_routes: Vec<RouteSnapshot>,
    pub selected_route: Option<RouteSnapshot>,
    pub gateway_check: GatewayCheck,
    pub dns_checks: Vec<DnsCheck>,
    pub http_checks: Vec<HttpCheck>,
    pub errors: Vec<ProbeError>,
}

pub fn select_default_route(routes: &[RouteSnapshot]) -> Option<RouteSnapshot> {
    routes
        .iter()
        .min_by_key(|route| {
            (
                !(route.adapter_is_up && route.adapter_is_ethernet),
                !route.adapter_is_up,
                route.combined_metric,
                route.interface_index,
            )
        })
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_network_diagnostic_status_contract() {
        let status = NetworkDiagnosticStatus {
            availability: RuntimeAvailability::Running,
            lifecycle: Some(DiagnosticLifecycle::Suspected),
            suspected_area: Some(DiagnosticArea::GatewayOrLocal),
            observed_at: Some("2026-06-23T00:00:00Z".into()),
            last_full_probe_at: None,
            evidence: vec![DiagnosticEvidence {
                source: EvidenceSource::Gateway,
                status: EvidenceStatus::Timeout,
                checked_at: Some("2026-06-23T00:00:00Z".into()),
                duration_ms: Some(1500),
                detail: Some("icmp_timeout".into()),
            }],
            error: None,
        };

        let json = serde_json::to_value(status).unwrap();
        assert_eq!(json["availability"], "running");
        assert_eq!(json["lifecycle"], "suspected");
        assert_eq!(json["suspectedArea"], "gateway_or_local");
        assert_eq!(json["evidence"][0]["source"], "gateway");
    }

    #[test]
    fn starting_status_has_no_invented_observation() {
        let status = NetworkDiagnosticStatus::starting();
        assert_eq!(status.availability, RuntimeAvailability::Starting);
        assert_eq!(status.lifecycle, None);
        assert_eq!(status.observed_at, None);
        assert!(status.evidence.is_empty());
    }

    fn route(index: u32, ethernet: bool, up: bool, metric: u32) -> RouteSnapshot {
        RouteSnapshot {
            interface_index: index,
            gateway: format!("192.168.0.{index}"),
            route_metric: metric,
            interface_metric: 0,
            combined_metric: metric,
            adapter_is_ethernet: ethernet,
            adapter_is_up: up,
        }
    }

    #[test]
    fn serializes_probe_status_as_snake_case() {
        assert_eq!(
            serde_json::to_string(&ProbeStatus::NotRun).unwrap(),
            "\"not_run\""
        );
    }

    #[test]
    fn selects_up_ethernet_before_lower_metric_non_ethernet() {
        let routes = vec![route(1, false, true, 1), route(2, true, true, 20)];

        assert_eq!(select_default_route(&routes).unwrap().interface_index, 2);
    }

    #[test]
    fn selects_lowest_metric_within_same_adapter_priority() {
        let routes = vec![route(1, true, true, 30), route(2, true, true, 10)];

        assert_eq!(select_default_route(&routes).unwrap().interface_index, 2);
    }

    #[test]
    fn returns_none_without_candidates() {
        assert!(select_default_route(&[]).is_none());
    }
}
