use super::{
    domain::{NetworkDiagnosticStatus, RuntimeAvailability},
    service::NetworkProbeCoordinator,
};
use std::{
    sync::{Arc, RwLock},
    thread::JoinHandle,
};

#[cfg(any(target_os = "windows", test))]
use super::{
    collector::{GOOGLE_URL, MICROSOFT_URL},
    domain::{DiagnosticArea, DiagnosticLifecycle},
    observation::assess,
    state_machine::NetworkDiagnosticStateMachine,
};
#[cfg(any(target_os = "windows", test))]
use std::{
    sync::{Condvar, Mutex},
    time::{Duration, Instant},
};

#[cfg(any(target_os = "windows", test))]
const BASELINE_INTERVAL: Duration = Duration::from_secs(10);
#[cfg(any(target_os = "windows", test))]
const EXTERNAL_INTERVAL: Duration = Duration::from_secs(20);
#[cfg(any(target_os = "windows", test))]
const FOCUSED_COOLDOWN: Duration = Duration::from_secs(30);

#[cfg(any(target_os = "windows", test))]
#[derive(Clone, Debug, PartialEq, Eq)]
enum ProbeRequest {
    Full,
    Baseline(Option<&'static str>),
    Focused,
}

#[cfg(any(target_os = "windows", test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WaitOutcome {
    Elapsed,
    Stopped,
}

#[cfg(any(target_os = "windows", test))]
trait RuntimeWait: Send {
    fn elapsed(&self) -> Duration;
    fn wait(&mut self, duration: Duration) -> WaitOutcome;
}

#[cfg(any(target_os = "windows", test))]
struct SystemWait {
    started: Instant,
    stop_state: Arc<(Mutex<bool>, Condvar)>,
}

#[cfg(any(target_os = "windows", test))]
impl SystemWait {
    fn new() -> (Self, StopHandle) {
        let stop_state = Arc::new((Mutex::new(false), Condvar::new()));
        (
            Self {
                started: Instant::now(),
                stop_state: stop_state.clone(),
            },
            StopHandle {
                stop_state: Some(stop_state),
            },
        )
    }
}

#[cfg(any(target_os = "windows", test))]
impl RuntimeWait for SystemWait {
    fn elapsed(&self) -> Duration {
        self.started.elapsed()
    }

    fn wait(&mut self, duration: Duration) -> WaitOutcome {
        let (lock, condition) = &*self.stop_state;
        let stopped = match lock.lock() {
            Ok(stopped) => stopped,
            Err(_) => return WaitOutcome::Stopped,
        };
        if *stopped {
            return WaitOutcome::Stopped;
        }
        match condition.wait_timeout_while(stopped, duration, |stopped| !*stopped) {
            Ok((stopped, _)) if *stopped => WaitOutcome::Stopped,
            Ok(_) => WaitOutcome::Elapsed,
            Err(_) => WaitOutcome::Stopped,
        }
    }
}

#[derive(Clone)]
struct StopHandle {
    #[cfg(any(target_os = "windows", test))]
    stop_state: Option<Arc<(Mutex<bool>, Condvar)>>,
}

impl StopHandle {
    fn inactive() -> Self {
        Self {
            #[cfg(any(target_os = "windows", test))]
            stop_state: None,
        }
    }

    fn stop(&self) {
        #[cfg(any(target_os = "windows", test))]
        {
            let Some(stop_state) = &self.stop_state else {
                return;
            };
            let (lock, condition) = &**stop_state;
            if let Ok(mut stopped) = lock.lock() {
                *stopped = true;
                condition.notify_all();
            }
        }
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub struct NetworkDiagnosticsRuntime {
    latest: Arc<RwLock<NetworkDiagnosticStatus>>,
    stop: StopHandle,
    worker: Option<JoinHandle<()>>,
}

#[cfg_attr(not(test), allow(dead_code))]
impl NetworkDiagnosticsRuntime {
    #[cfg(target_os = "windows")]
    pub fn platform(coordinator: NetworkProbeCoordinator) -> Self {
        let (wait, stop) = SystemWait::new();
        Self::start(coordinator, wait, stop)
    }

    #[cfg(not(target_os = "windows"))]
    pub fn platform(_coordinator: NetworkProbeCoordinator) -> Self {
        Self::unavailable()
    }

    #[cfg(target_os = "windows")]
    fn start<W: RuntimeWait + 'static>(
        coordinator: NetworkProbeCoordinator,
        wait: W,
        stop: StopHandle,
    ) -> Self {
        let latest = Arc::new(RwLock::new(NetworkDiagnosticStatus::starting()));
        let worker_latest = latest.clone();
        let worker = std::thread::spawn(move || run_worker(coordinator, wait, worker_latest));
        Self {
            latest,
            stop,
            worker: Some(worker),
        }
    }

    pub fn status(&self) -> Result<NetworkDiagnosticStatus, String> {
        self.latest
            .read()
            .map(|status| status.clone())
            .map_err(|_| "network diagnostic status lock failed".into())
    }

    #[cfg(test)]
    pub fn unavailable_for_test() -> Self {
        Self::unavailable()
    }

    fn unavailable() -> Self {
        let mut status = NetworkDiagnosticStatus::starting();
        status.availability = RuntimeAvailability::Unavailable;
        Self {
            latest: Arc::new(RwLock::new(status)),
            stop: StopHandle::inactive(),
            worker: None,
        }
    }

    fn apply_runtime_error(status: &mut NetworkDiagnosticStatus, message: String) {
        status.availability = RuntimeAvailability::Error;
        status.error = Some(super::domain::ProbeError {
            stage: "runtime".into(),
            code: "runtime_failed".into(),
            message,
            native_code: None,
        });
    }
}

impl Drop for NetworkDiagnosticsRuntime {
    fn drop(&mut self) {
        self.stop.stop();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

#[cfg(any(target_os = "windows", test))]
fn run_worker<W: RuntimeWait>(
    coordinator: NetworkProbeCoordinator,
    mut wait: W,
    latest: Arc<RwLock<NetworkDiagnosticStatus>>,
) {
    let mut machine = NetworkDiagnosticStateMachine::new();
    let mut baseline_count = 0_u64;
    let mut external_index = 0_usize;
    let mut last_focused = None;

    if !run_probe(&coordinator, &mut machine, &latest, ProbeRequest::Full) {
        return;
    }
    if focused_needed(&machine.status()) {
        if !run_probe(&coordinator, &mut machine, &latest, ProbeRequest::Focused) {
            return;
        }
        last_focused = Some(wait.elapsed());
    }

    loop {
        if wait.wait(BASELINE_INTERVAL) == WaitOutcome::Stopped {
            return;
        }
        baseline_count += 1;
        let request = baseline_request(baseline_count, external_index);
        let external_due = matches!(request, ProbeRequest::Baseline(Some(_)));
        if !run_probe(&coordinator, &mut machine, &latest, request) {
            return;
        }
        if external_due {
            external_index = (external_index + 1) % 2;
        }

        let now = wait.elapsed();
        let cooldown_elapsed =
            last_focused.is_none_or(|last| now.saturating_sub(last) >= FOCUSED_COOLDOWN);
        if cooldown_elapsed && focused_needed(&machine.status()) {
            if !run_probe(&coordinator, &mut machine, &latest, ProbeRequest::Focused) {
                return;
            }
            last_focused = Some(wait.elapsed());
        }
    }
}

#[cfg(any(target_os = "windows", test))]
fn baseline_request(baseline_count: u64, external_index: usize) -> ProbeRequest {
    let external_due =
        (baseline_count * BASELINE_INTERVAL.as_secs()).is_multiple_of(EXTERNAL_INTERVAL.as_secs());
    ProbeRequest::Baseline(external_due.then_some([MICROSOFT_URL, GOOGLE_URL][external_index]))
}

#[cfg(any(target_os = "windows", test))]
fn run_probe(
    coordinator: &NetworkProbeCoordinator,
    machine: &mut NetworkDiagnosticStateMachine,
    latest: &Arc<RwLock<NetworkDiagnosticStatus>>,
    request: ProbeRequest,
) -> bool {
    let result = match request {
        ProbeRequest::Full | ProbeRequest::Focused => coordinator.collect_full(),
        ProbeRequest::Baseline(endpoint) => coordinator.collect_baseline(endpoint),
    };
    let snapshot = match result {
        Ok(snapshot) => snapshot,
        Err(message) => {
            store_runtime_error(latest, message);
            return false;
        }
    };
    let observed_at = snapshot.collected_at.clone();
    let full = matches!(request, ProbeRequest::Full | ProbeRequest::Focused);
    let status = machine.apply(assess(snapshot), &observed_at, full);
    match latest.write() {
        Ok(mut latest) => {
            *latest = status;
            true
        }
        Err(_) => false,
    }
}

#[cfg(any(target_os = "windows", test))]
fn focused_needed(status: &NetworkDiagnosticStatus) -> bool {
    if status.lifecycle == Some(DiagnosticLifecycle::Suspected) {
        return true;
    }
    matches!(
        (&status.lifecycle, &status.suspected_area),
        (
            Some(DiagnosticLifecycle::Incident | DiagnosticLifecycle::Recovering),
            Some(DiagnosticArea::Dns | DiagnosticArea::External | DiagnosticArea::Unknown)
        )
    )
}

#[cfg(any(target_os = "windows", test))]
fn store_runtime_error(latest: &Arc<RwLock<NetworkDiagnosticStatus>>, message: String) {
    if let Ok(mut status) = latest.write() {
        NetworkDiagnosticsRuntime::apply_runtime_error(&mut status, message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::network::{
        collector::{NetworkCollector, NetworkInventory},
        domain::{
            AdapterSnapshot, DnsCheck, GatewayCheck, HttpCheck, ProbeError, ProbeStatus,
            RouteSnapshot,
        },
        service::NetworkProbeService,
    };
    use std::{
        collections::VecDeque,
        sync::{
            atomic::{AtomicUsize, Ordering},
            Barrier,
        },
        thread,
    };

    #[derive(Clone)]
    struct ScriptedWait {
        elapsed: Duration,
        outcomes: VecDeque<WaitOutcome>,
    }

    impl ScriptedWait {
        fn elapsed(count: usize) -> Self {
            Self {
                elapsed: Duration::ZERO,
                outcomes: std::iter::repeat_n(WaitOutcome::Elapsed, count)
                    .chain([WaitOutcome::Stopped])
                    .collect(),
            }
        }
    }

    impl RuntimeWait for ScriptedWait {
        fn elapsed(&self) -> Duration {
            self.elapsed
        }

        fn wait(&mut self, duration: Duration) -> WaitOutcome {
            let outcome = self.outcomes.pop_front().unwrap_or(WaitOutcome::Stopped);
            if outcome == WaitOutcome::Elapsed {
                self.elapsed += duration;
            }
            outcome
        }
    }

    struct RecordingCollector {
        probes: Arc<Mutex<Vec<ProbeRequest>>>,
        gateway_status: ProbeStatus,
        unknown_inventory_failure: bool,
        failed_endpoint: Option<&'static str>,
        active: Option<Arc<AtomicUsize>>,
        maximum_active: Option<Arc<AtomicUsize>>,
        first_call_barrier: Option<Arc<Barrier>>,
        collection_calls: Option<Arc<AtomicUsize>>,
    }

    impl RecordingCollector {
        fn normal(probes: Arc<Mutex<Vec<ProbeRequest>>>) -> Self {
            Self {
                probes,
                gateway_status: ProbeStatus::Success,
                unknown_inventory_failure: false,
                failed_endpoint: None,
                active: None,
                maximum_active: None,
                first_call_barrier: None,
                collection_calls: None,
            }
        }

        fn unknown_failure(probes: Arc<Mutex<Vec<ProbeRequest>>>) -> Self {
            Self {
                unknown_inventory_failure: true,
                ..Self::normal(probes)
            }
        }
    }

    impl NetworkCollector for RecordingCollector {
        fn collector_name(&self) -> &'static str {
            "runtime-fake"
        }

        fn collect_inventory(&self) -> NetworkInventory {
            if let (Some(active), Some(maximum)) = (&self.active, &self.maximum_active) {
                let current = active.fetch_add(1, Ordering::SeqCst) + 1;
                maximum.fetch_max(current, Ordering::SeqCst);
                if self
                    .collection_calls
                    .as_ref()
                    .is_some_and(|calls| calls.fetch_add(1, Ordering::SeqCst) == 0)
                {
                    self.first_call_barrier.as_ref().unwrap().wait();
                }
                active.fetch_sub(1, Ordering::SeqCst);
            }
            if self.unknown_inventory_failure {
                return NetworkInventory {
                    adapters: vec![],
                    default_routes: vec![],
                    errors: vec![ProbeError {
                        stage: "inventory".into(),
                        code: "inventory_failed".into(),
                        message: "inventory unavailable".into(),
                        native_code: None,
                    }],
                };
            }
            NetworkInventory {
                adapters: vec![AdapterSnapshot {
                    name: "ethernet".into(),
                    friendly_name: "Ethernet".into(),
                    interface_index: 1,
                    if_type: 6,
                    operational_status: "up".into(),
                    mac_address: None,
                    ipv4_addresses: vec!["192.168.0.2".into()],
                }],
                default_routes: vec![RouteSnapshot {
                    interface_index: 1,
                    gateway: "192.168.0.1".into(),
                    route_metric: 1,
                    interface_metric: 1,
                    combined_metric: 2,
                    adapter_is_ethernet: true,
                    adapter_is_up: true,
                }],
                errors: vec![],
            }
        }

        fn check_gateway(&self, _route: Option<&RouteSnapshot>) -> GatewayCheck {
            GatewayCheck {
                status: self.gateway_status.clone(),
                duration_ms: 0,
                reply_address: None,
                round_trip_ms: None,
                error: None,
            }
        }

        fn check_dns(&self) -> Vec<DnsCheck> {
            self.probes.lock().unwrap().push(ProbeRequest::Full);
            vec![
                dns("www.msftconnecttest.com"),
                dns("connectivitycheck.gstatic.com"),
            ]
        }

        fn check_http_endpoint(&self, url: &str) -> HttpCheck {
            self.probes
                .lock()
                .unwrap()
                .push(ProbeRequest::Baseline(Some(if url == MICROSOFT_URL {
                    MICROSOFT_URL
                } else {
                    GOOGLE_URL
                })));
            let mut result = http(url);
            if self.failed_endpoint == Some(url) {
                result.status = ProbeStatus::Timeout;
                result.status_code = None;
                result.body_matches = None;
            }
            result
        }

        fn check_http(&self) -> Vec<HttpCheck> {
            vec![http(MICROSOFT_URL), http(GOOGLE_URL)]
        }
    }

    fn dns(hostname: &str) -> DnsCheck {
        DnsCheck {
            hostname: hostname.into(),
            status: ProbeStatus::Success,
            duration_ms: 0,
            addresses: vec!["1.1.1.1".into()],
            error: None,
        }
    }

    fn http(url: &str) -> HttpCheck {
        HttpCheck {
            url: url.into(),
            status: ProbeStatus::Success,
            duration_ms: 0,
            status_code: Some(200),
            body_matches: Some(true),
            error: None,
        }
    }

    fn coordinator(collector: RecordingCollector) -> NetworkProbeCoordinator {
        NetworkProbeCoordinator::from_service_for_test(NetworkProbeService::new(Box::new(
            collector,
        )))
    }

    fn run(collector: RecordingCollector, waits: usize) -> NetworkDiagnosticStatus {
        let latest = Arc::new(RwLock::new(NetworkDiagnosticStatus::starting()));
        run_worker(
            coordinator(collector),
            ScriptedWait::elapsed(waits),
            latest.clone(),
        );
        let status = latest.read().unwrap().clone();
        status
    }

    #[test]
    fn startup_is_full_then_external_targets_alternate_every_two_baselines() {
        let mut probes = vec![ProbeRequest::Full];
        let mut external_index = 0;
        for baseline_count in 1..=5 {
            let request = baseline_request(baseline_count, external_index);
            if matches!(request, ProbeRequest::Baseline(Some(_))) {
                external_index = (external_index + 1) % 2;
            }
            probes.push(request);
        }
        assert_eq!(
            probes,
            vec![
                ProbeRequest::Full,
                ProbeRequest::Baseline(None),
                ProbeRequest::Baseline(Some(MICROSOFT_URL)),
                ProbeRequest::Baseline(None),
                ProbeRequest::Baseline(Some(GOOGLE_URL)),
                ProbeRequest::Baseline(None),
            ]
        );
    }

    #[test]
    fn persistent_anomaly_focuses_at_startup_and_no_more_often_than_thirty_seconds() {
        let probes = Arc::new(Mutex::new(vec![]));
        run(RecordingCollector::unknown_failure(probes.clone()), 4);

        let full_count = probes
            .lock()
            .unwrap()
            .iter()
            .filter(|request| request == &&ProbeRequest::Full)
            .count();
        // The startup full is the first assessment; focused runs at t=0 and t=30.
        assert_eq!(full_count, 3);
    }

    #[test]
    fn external_target_advances_after_a_failed_attempt() {
        let probes = Arc::new(Mutex::new(vec![]));
        let collector = RecordingCollector {
            failed_endpoint: Some(MICROSOFT_URL),
            ..RecordingCollector::normal(probes.clone())
        };
        run(collector, 4);

        let external_baselines: Vec<_> = probes
            .lock()
            .unwrap()
            .iter()
            .filter(|request| matches!(request, ProbeRequest::Baseline(Some(_))))
            .cloned()
            .collect();
        assert_eq!(
            external_baselines,
            vec![
                ProbeRequest::Baseline(Some(MICROSOFT_URL)),
                ProbeRequest::Baseline(Some(GOOGLE_URL)),
            ]
        );
    }

    #[test]
    fn remote_incidents_need_periodic_focused_recovery_checks_but_local_incidents_do_not() {
        for area in [
            DiagnosticArea::Dns,
            DiagnosticArea::External,
            DiagnosticArea::Unknown,
        ] {
            let mut status = NetworkDiagnosticStatus::starting();
            status.lifecycle = Some(DiagnosticLifecycle::Incident);
            status.suspected_area = Some(area);
            assert!(focused_needed(&status));
        }
        for area in [
            DiagnosticArea::LocalConnection,
            DiagnosticArea::GatewayOrLocal,
        ] {
            let mut status = NetworkDiagnosticStatus::starting();
            status.lifecycle = Some(DiagnosticLifecycle::Incident);
            status.suspected_area = Some(area);
            assert!(!focused_needed(&status));
        }
    }

    #[test]
    fn stop_interrupts_wait_without_another_probe() {
        let probes = Arc::new(Mutex::new(vec![]));
        run(RecordingCollector::normal(probes.clone()), 0);
        assert_eq!(*probes.lock().unwrap(), vec![ProbeRequest::Full]);
    }

    #[test]
    fn stop_handle_interrupts_the_system_wait() {
        let (mut wait, stop) = SystemWait::new();
        stop.stop();

        assert_eq!(wait.wait(Duration::from_secs(3_600)), WaitOutcome::Stopped);
    }

    #[test]
    fn coordinator_serializes_manual_and_runtime_collection() {
        let active = Arc::new(AtomicUsize::new(0));
        let maximum = Arc::new(AtomicUsize::new(0));
        let first_call_barrier = Arc::new(Barrier::new(2));
        let collector = RecordingCollector {
            active: Some(active),
            maximum_active: Some(maximum.clone()),
            first_call_barrier: Some(first_call_barrier.clone()),
            collection_calls: Some(Arc::new(AtomicUsize::new(0))),
            ..RecordingCollector::normal(Arc::new(Mutex::new(vec![])))
        };
        let coordinator = coordinator(collector);
        let manual = coordinator.clone();
        let automatic = coordinator.clone();
        let first = thread::spawn(move || manual.collect_full().unwrap());
        first_call_barrier.wait();
        let second = thread::spawn(move || automatic.collect_baseline(None).unwrap());
        first.join().unwrap();
        second.join().unwrap();
        assert_eq!(maximum.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn automatic_http_requests_stay_within_hourly_limits() {
        let steady_state_baselines = 3_600 / BASELINE_INTERVAL.as_secs();
        let normal_http = steady_state_baselines / 2;
        let persistent_focused_http = (3_600 / FOCUSED_COOLDOWN.as_secs()) * 2;

        // Startup is outside this steady-state hourly budget.
        assert_eq!(normal_http, 180);
        assert!(normal_http + persistent_focused_http <= 420);
    }

    #[test]
    fn collection_failure_is_exposed_and_stops_the_worker() {
        let coordinator = NetworkProbeCoordinator::failed_for_test("collector exploded");
        let latest = Arc::new(RwLock::new(NetworkDiagnosticStatus::starting()));
        run_worker(coordinator, ScriptedWait::elapsed(5), latest.clone());
        let status = latest.read().unwrap();
        assert_eq!(status.availability, RuntimeAvailability::Error);
        assert_eq!(status.error.as_ref().unwrap().stage, "runtime");
        assert_eq!(status.error.as_ref().unwrap().code, "runtime_failed");
        assert_eq!(status.error.as_ref().unwrap().message, "collector exploded");
        assert_eq!(status.error.as_ref().unwrap().native_code, None);
    }

    #[test]
    fn unavailable_runtime_starts_no_worker() {
        let runtime = NetworkDiagnosticsRuntime::unavailable_for_test();
        assert_eq!(
            runtime.status().unwrap().availability,
            RuntimeAvailability::Unavailable
        );
        assert!(runtime.worker.is_none());

        let platform = NetworkDiagnosticsRuntime::platform(
            NetworkProbeCoordinator::failed_for_test("must not collect"),
        );
        assert_eq!(
            platform.status().unwrap().availability,
            RuntimeAvailability::Unavailable
        );
        assert!(platform.worker.is_none());
    }

    #[test]
    fn status_reports_the_exact_lock_failure_text() {
        let latest = Arc::new(RwLock::new(NetworkDiagnosticStatus::starting()));
        let poison = latest.clone();
        let _ = thread::spawn(move || {
            let _guard = poison.write().unwrap();
            panic!("poison status lock");
        })
        .join();
        let runtime = NetworkDiagnosticsRuntime {
            latest,
            stop: StopHandle::inactive(),
            worker: None,
        };

        assert_eq!(
            runtime.status().unwrap_err(),
            "network diagnostic status lock failed"
        );
    }
}
