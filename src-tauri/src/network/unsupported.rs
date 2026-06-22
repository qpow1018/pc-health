use super::{
    collector::{
        NetworkCollector, NetworkInventory, GOOGLE_HOSTNAME, GOOGLE_URL, MICROSOFT_HOSTNAME,
        MICROSOFT_URL,
    },
    domain::{DnsCheck, GatewayCheck, HttpCheck, ProbeError, ProbeStatus, RouteSnapshot},
};

pub struct UnsupportedCollector;

impl UnsupportedCollector {
    fn error(stage: &str) -> ProbeError {
        ProbeError {
            stage: stage.into(),
            code: "platform_unsupported".into(),
            message: "network probes are only available on Windows".into(),
            native_code: None,
        }
    }
}

impl NetworkCollector for UnsupportedCollector {
    fn collector_name(&self) -> &'static str {
        "unsupported"
    }

    fn collect_inventory(&self) -> NetworkInventory {
        NetworkInventory {
            adapters: vec![],
            default_routes: vec![],
            errors: vec![Self::error("inventory")],
        }
    }

    fn check_gateway(&self, _route: Option<&RouteSnapshot>) -> GatewayCheck {
        GatewayCheck {
            status: ProbeStatus::Unsupported,
            duration_ms: 0,
            reply_address: None,
            round_trip_ms: None,
            error: Some(Self::error("gateway")),
        }
    }

    fn check_dns(&self) -> Vec<DnsCheck> {
        [MICROSOFT_HOSTNAME, GOOGLE_HOSTNAME]
            .into_iter()
            .map(|hostname| DnsCheck {
                hostname: hostname.into(),
                status: ProbeStatus::Unsupported,
                duration_ms: 0,
                addresses: vec![],
                error: Some(Self::error("system_name_resolution")),
            })
            .collect()
    }

    fn check_http(&self) -> Vec<HttpCheck> {
        [MICROSOFT_URL, GOOGLE_URL]
            .into_iter()
            .map(|url| HttpCheck {
                url: url.into(),
                status: ProbeStatus::Unsupported,
                duration_ms: 0,
                status_code: None,
                body_matches: None,
                error: Some(Self::error("http")),
            })
            .collect()
    }
}
