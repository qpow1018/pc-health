use super::{
    collector::NetworkCollector,
    domain::{select_default_route, NetworkProbeSnapshot},
    unsupported::UnsupportedCollector,
};
use chrono::Utc;
use std::time::Instant;

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

    pub fn collect(&self) -> NetworkProbeSnapshot {
        let started = Instant::now();
        let inventory = self.collector.collect_inventory();
        let selected_route = select_default_route(&inventory.default_routes);
        let gateway_check = self.collector.check_gateway(selected_route.as_ref());
        let dns_checks = self.collector.check_dns();
        let http_checks = self.collector.check_http();

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

fn duration_ms(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::{
        collector::{NetworkCollector, NetworkInventory},
        domain::{
            DnsCheck, GatewayCheck, HttpCheck, ProbeError, ProbeStatus, RouteSnapshot,
        },
    };
    use std::sync::{Arc, Mutex};

    struct FakeCollector {
        gateway_route: Arc<Mutex<Option<RouteSnapshot>>>,
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
            vec![]
        }

        fn check_http(&self) -> Vec<HttpCheck> {
            vec![]
        }
    }

    #[test]
    fn preserves_partial_results_and_passes_selected_route_to_gateway_check() {
        let gateway_route = Arc::new(Mutex::new(None));
        let service = NetworkProbeService::new(Box::new(FakeCollector {
            gateway_route: gateway_route.clone(),
        }));

        let snapshot = service.collect();

        assert_eq!(snapshot.collector, "fake");
        assert_eq!(snapshot.errors[0].stage, "adapters");
        assert_eq!(snapshot.gateway_check.status, ProbeStatus::NotRun);
        assert_eq!(snapshot.selected_route.as_ref().unwrap().interface_index, 7);
        assert_eq!(
            gateway_route.lock().unwrap().as_ref().unwrap().interface_index,
            7
        );
    }
}
