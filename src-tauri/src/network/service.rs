use super::{
    collector::NetworkCollector,
    domain::{select_default_route, NetworkProbeSnapshot},
};
use chrono::Utc;
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

#[cfg(not(target_os = "windows"))]
use super::unsupported::UnsupportedCollector;
#[cfg(target_os = "windows")]
use super::windows::WindowsCollector;

pub struct NetworkProbeService {
    collector: Box<dyn NetworkCollector>,
}

impl NetworkProbeService {
    pub fn new(collector: Box<dyn NetworkCollector>) -> Self {
        Self { collector }
    }

    #[cfg(not(target_os = "windows"))]
    pub fn platform() -> Self {
        Self::new(Box::new(UnsupportedCollector))
    }

    #[cfg(target_os = "windows")]
    pub fn platform() -> Self {
        Self::new(Box::new(WindowsCollector))
    }

    #[cfg(test)]
    pub fn collect(&self) -> NetworkProbeSnapshot {
        self.collect_full()
    }

    pub fn collect_full(&self) -> NetworkProbeSnapshot {
        self.collect_with(None, true)
    }

    #[cfg_attr(not(target_os = "windows"), allow(dead_code))]
    pub fn collect_baseline(&self, http_url: Option<&str>) -> NetworkProbeSnapshot {
        self.collect_with(http_url, false)
    }

    fn collect_with(&self, http_url: Option<&str>, full: bool) -> NetworkProbeSnapshot {
        let started = Instant::now();
        let inventory = self.collector.collect_inventory();
        let selected_route = select_default_route(&inventory.default_routes);
        let gateway_check = self.collector.check_gateway(selected_route.as_ref());
        let dns_checks = if full {
            self.collector.check_dns()
        } else {
            vec![]
        };
        let http_checks = if full {
            self.collector.check_http()
        } else {
            http_url
                .map(|url| vec![self.collector.check_http_endpoint(url)])
                .unwrap_or_default()
        };

        NetworkProbeSnapshot {
            collected_at: Utc::now().to_rfc3339(),
            collector: self.collector.collector_name().into(),
            duration_ms: duration_ms(started),
            adapters: inventory.adapters,
            default_routes: inventory.default_routes,
            selected_route,
            gateway_check,
            dns_checks,
            http_checks,
            errors: inventory.errors,
        }
    }
}

#[derive(Clone)]
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub struct NetworkProbeCoordinator {
    service: Arc<Mutex<NetworkProbeService>>,
    #[cfg(test)]
    failure: Option<String>,
}

#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
impl NetworkProbeCoordinator {
    pub fn platform() -> Self {
        Self {
            service: Arc::new(Mutex::new(NetworkProbeService::platform())),
            #[cfg(test)]
            failure: None,
        }
    }

    pub fn collect_full(&self) -> Result<NetworkProbeSnapshot, String> {
        #[cfg(test)]
        if let Some(message) = &self.failure {
            return Err(message.clone());
        }
        self.service
            .lock()
            .map_err(|_| "network probe coordinator lock failed".to_string())
            .map(|service| service.collect_full())
    }

    pub fn collect_baseline(&self, url: Option<&str>) -> Result<NetworkProbeSnapshot, String> {
        #[cfg(test)]
        if let Some(message) = &self.failure {
            return Err(message.clone());
        }
        self.service
            .lock()
            .map_err(|_| "network probe coordinator lock failed".to_string())
            .map(|service| service.collect_baseline(url))
    }

    #[cfg(test)]
    pub(crate) fn from_service_for_test(service: NetworkProbeService) -> Self {
        Self {
            service: Arc::new(Mutex::new(service)),
            failure: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn failed_for_test(message: &str) -> Self {
        Self {
            service: Arc::new(Mutex::new(NetworkProbeService::new(Box::new(
                FailingCollector,
            )))),
            failure: Some(message.into()),
        }
    }
}

#[cfg(test)]
struct FailingCollector;

#[cfg(test)]
impl NetworkCollector for FailingCollector {
    fn collector_name(&self) -> &'static str {
        "failing"
    }
    fn collect_inventory(&self) -> super::collector::NetworkInventory {
        unreachable!()
    }
    fn check_gateway(
        &self,
        _: Option<&super::domain::RouteSnapshot>,
    ) -> super::domain::GatewayCheck {
        unreachable!()
    }
    fn check_dns(&self) -> Vec<super::domain::DnsCheck> {
        unreachable!()
    }
    fn check_http_endpoint(&self, _: &str) -> super::domain::HttpCheck {
        unreachable!()
    }
    fn check_http(&self) -> Vec<super::domain::HttpCheck> {
        unreachable!()
    }
}

fn duration_ms(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::{
        collector::{NetworkCollector, NetworkInventory, GOOGLE_URL, MICROSOFT_URL},
        domain::{DnsCheck, GatewayCheck, HttpCheck, ProbeError, ProbeStatus, RouteSnapshot},
    };
    use std::sync::{Arc, Mutex};

    struct FakeCollector {
        gateway_route: Arc<Mutex<Option<RouteSnapshot>>>,
        http_calls: Arc<Mutex<Vec<String>>>,
    }

    impl FakeCollector {
        fn recording(http_calls: Arc<Mutex<Vec<String>>>) -> Self {
            Self {
                gateway_route: Arc::new(Mutex::new(None)),
                http_calls,
            }
        }

        fn http_check(url: &str) -> HttpCheck {
            HttpCheck {
                url: url.into(),
                status: ProbeStatus::Success,
                duration_ms: 0,
                status_code: Some(200),
                body_matches: Some(true),
                error: None,
            }
        }
    }

    impl Default for FakeCollector {
        fn default() -> Self {
            Self::recording(Arc::new(Mutex::new(vec![])))
        }
    }

    impl NetworkCollector for FakeCollector {
        fn collector_name(&self) -> &'static str {
            "fake"
        }

        fn collect_inventory(&self) -> NetworkInventory {
            NetworkInventory {
                adapters: vec![],
                default_routes: vec![RouteSnapshot {
                    interface_index: 7,
                    gateway: "192.168.0.1".into(),
                    route_metric: 10,
                    interface_metric: 5,
                    combined_metric: 15,
                    adapter_is_ethernet: true,
                    adapter_is_up: true,
                }],
                errors: vec![ProbeError {
                    stage: "adapters".into(),
                    code: "partial_failure".into(),
                    message: "one adapter could not be read".into(),
                    native_code: Some(13),
                }],
            }
        }

        fn check_gateway(&self, route: Option<&RouteSnapshot>) -> GatewayCheck {
            *self.gateway_route.lock().unwrap() = route.cloned();
            GatewayCheck {
                status: ProbeStatus::NotRun,
                duration_ms: 0,
                reply_address: None,
                round_trip_ms: None,
                error: None,
            }
        }

        fn check_dns(&self) -> Vec<DnsCheck> {
            ["microsoft", "google"]
                .into_iter()
                .map(|hostname| DnsCheck {
                    hostname: hostname.into(),
                    status: ProbeStatus::Success,
                    duration_ms: 0,
                    addresses: vec![],
                    error: None,
                })
                .collect()
        }

        fn check_http_endpoint(&self, url: &str) -> HttpCheck {
            self.http_calls.lock().unwrap().push(url.into());
            Self::http_check(url)
        }

        fn check_http(&self) -> Vec<HttpCheck> {
            [MICROSOFT_URL, GOOGLE_URL]
                .into_iter()
                .map(|url| {
                    self.http_calls.lock().unwrap().push(url.into());
                    Self::http_check(url)
                })
                .collect()
        }
    }

    #[test]
    fn preserves_partial_results_and_passes_selected_route_to_gateway_check() {
        let gateway_route = Arc::new(Mutex::new(None));
        let service = NetworkProbeService::new(Box::new(FakeCollector {
            gateway_route: gateway_route.clone(),
            http_calls: Arc::new(Mutex::new(vec![])),
        }));

        let snapshot = service.collect();

        assert_eq!(snapshot.collector, "fake");
        assert_eq!(snapshot.errors[0].stage, "adapters");
        assert_eq!(snapshot.gateway_check.status, ProbeStatus::NotRun);
        assert_eq!(snapshot.selected_route.as_ref().unwrap().interface_index, 7);
        assert_eq!(
            gateway_route
                .lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .interface_index,
            7
        );
    }

    #[test]
    fn baseline_collects_inventory_gateway_and_only_requested_http_endpoint() {
        let calls = Arc::new(Mutex::new(vec![]));
        let service = NetworkProbeService::new(Box::new(FakeCollector::recording(calls.clone())));

        let snapshot = service.collect_baseline(Some(MICROSOFT_URL));

        assert!(snapshot.dns_checks.is_empty());
        assert_eq!(snapshot.http_checks.len(), 1);
        assert_eq!(snapshot.http_checks[0].url, MICROSOFT_URL);
        assert_eq!(*calls.lock().unwrap(), vec![MICROSOFT_URL]);
    }

    #[test]
    fn full_probe_keeps_both_dns_and_http_targets() {
        let service = NetworkProbeService::new(Box::new(FakeCollector::default()));

        let snapshot = service.collect_full();

        assert_eq!(snapshot.dns_checks.len(), 2);
        assert_eq!(snapshot.http_checks.len(), 2);
    }
}
