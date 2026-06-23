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

    #[cfg(any(target_os = "windows", test))]
    fn start<W: RuntimeWait + 'static>(
        coordinator: NetworkProbeCoordinator,
        wait: W,
        stop: StopHandle,
    ) -> Self {
        let latest = Arc::new(RwLock::new(NetworkDiagnosticStatus::starting()));
        let worker_latest = latest.clone();
        let panic_latest = latest.clone();
        let worker = std::thread::spawn(move || {
            if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                run_worker(coordinator, wait, worker_latest);
            }))
            .is_err()
            {
                store_runtime_error(&panic_latest, "network diagnostic worker panicked".into());
            }
        });
        Self {
            latest,
            stop,
            worker: Some(worker),
        }
    }

    pub fn status(&self) -> Result<NetworkDiagnosticStatus, String> {
        match self.latest.read() {
            Ok(status) => Ok(status.clone()),
            Err(poisoned) => {
                let mut status = poisoned.into_inner().clone();
                Self::apply_runtime_error(
                    &mut status,
                    "network diagnostic status lock failed".into(),
                );
                Ok(status)
            }
        }
    }

    #[cfg(test)]
    pub fn unavailable_for_test() -> Self {
        Self::unavailable()
    }

    #[cfg(test)]
    fn start_for_test<W: RuntimeWait + 'static>(
        coordinator: NetworkProbeCoordinator,
        wait: W,
    ) -> Self {
        Self::start(coordinator, wait, StopHandle::inactive())
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
    wait: W,
    latest: Arc<RwLock<NetworkDiagnosticStatus>>,
) {
    run_worker_observed(coordinator, wait, latest, |_, _| {});
}

#[cfg(any(target_os = "windows", test))]
fn run_worker_observed<W, F>(
    coordinator: NetworkProbeCoordinator,
    mut wait: W,
    latest: Arc<RwLock<NetworkDiagnosticStatus>>,
    mut observe: F,
) where
    W: RuntimeWait,
    F: FnMut(&ProbeRequest, Duration),
{
    let mut machine = NetworkDiagnosticStateMachine::new();
    let mut baseline_count = 0_u64;
    let mut external_index = 0_usize;
    let mut last_focused = None;

    let startup = ProbeRequest::Full;
    observe(&startup, wait.elapsed());
    if !run_probe(&coordinator, &mut machine, &latest, startup) {
        return;
    }
    if focused_needed(&machine.status()) {
        let focused = ProbeRequest::Focused;
        observe(&focused, wait.elapsed());
        if !run_probe(&coordinator, &mut machine, &latest, focused) {
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
        observe(&request, wait.elapsed());
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
            let focused = ProbeRequest::Focused;
            observe(&focused, wait.elapsed());
            if !run_probe(&coordinator, &mut machine, &latest, focused) {
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
        Err(poisoned) => {
            let mut latest = poisoned.into_inner();
            NetworkDiagnosticsRuntime::apply_runtime_error(
                &mut latest,
                "network diagnostic status lock failed".into(),
            );
            false
        }
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
    match latest.write() {
        Ok(mut status) => NetworkDiagnosticsRuntime::apply_runtime_error(&mut status, message),
        Err(poisoned) => {
            let mut status = poisoned.into_inner();
            NetworkDiagnosticsRuntime::apply_runtime_error(&mut status, message);
        }
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
        elapsed: Arc<Mutex<Duration>>,
        outcomes: VecDeque<WaitOutcome>,
        wait_log: Arc<Mutex<Vec<Duration>>>,
    }

    impl ScriptedWait {
        fn elapsed(count: usize) -> Self {
            Self {
                elapsed: Arc::new(Mutex::new(Duration::ZERO)),
                outcomes: std::iter::repeat_n(WaitOutcome::Elapsed, count)
                    .chain([WaitOutcome::Stopped])
                    .collect(),
                wait_log: Arc::new(Mutex::new(vec![])),
            }
        }
    }

    impl RuntimeWait for ScriptedWait {
        fn elapsed(&self) -> Duration {
            *self.elapsed.lock().unwrap()
        }

        fn wait(&mut self, duration: Duration) -> WaitOutcome {
            self.wait_log.lock().unwrap().push(duration);
            let outcome = self.outcomes.pop_front().unwrap_or(WaitOutcome::Stopped);
            if outcome == WaitOutcome::Elapsed {
                *self.elapsed.lock().unwrap() += duration;
            }
            outcome
        }
    }

    #[derive(Clone, Copy)]
    enum Scenario {
        Normal,
        Unknown,
        Local,
        Gateway,
        Dns,
        External,
    }

    struct RecordingCollector {
        probes: Arc<Mutex<Vec<ProbeRequest>>>,
        scenario: Scenario,
        failed_endpoint: Option<&'static str>,
        active: Option<Arc<AtomicUsize>>,
        maximum_active: Option<Arc<AtomicUsize>>,
        first_call_barrier: Option<Arc<Barrier>>,
        collection_calls: Option<Arc<AtomicUsize>>,
        fake_clock: Option<Arc<Mutex<Duration>>>,
        slow_by: Duration,
    }

    impl RecordingCollector {
        fn normal(probes: Arc<Mutex<Vec<ProbeRequest>>>) -> Self {
            Self {
                probes,
                scenario: Scenario::Normal,
                failed_endpoint: None,
                active: None,
                maximum_active: None,
                first_call_barrier: None,
                collection_calls: None,
                fake_clock: None,
                slow_by: Duration::ZERO,
            }
        }

        fn unknown_failure(probes: Arc<Mutex<Vec<ProbeRequest>>>) -> Self {
            Self {
                scenario: Scenario::Unknown,
                ..Self::normal(probes)
            }
        }

        fn scenario(scenario: Scenario) -> Self {
            Self {
                scenario,
                ..Self::normal(Arc::new(Mutex::new(vec![])))
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
            if let Some(clock) = &self.fake_clock {
                *clock.lock().unwrap() += self.slow_by;
            }
            if matches!(self.scenario, Scenario::Unknown) {
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
            if matches!(self.scenario, Scenario::Local) {
                return NetworkInventory {
                    adapters: vec![],
                    default_routes: vec![],
                    errors: vec![],
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
                status: if matches!(self.scenario, Scenario::Gateway) {
                    ProbeStatus::Timeout
                } else {
                    ProbeStatus::Success
                },
                duration_ms: 0,
                reply_address: None,
                round_trip_ms: None,
                error: None,
            }
        }

        fn check_dns(&self) -> Vec<DnsCheck> {
            self.probes.lock().unwrap().push(ProbeRequest::Full);
            let mut checks = vec![
                dns("www.msftconnecttest.com"),
                dns("connectivitycheck.gstatic.com"),
            ];
            if matches!(self.scenario, Scenario::Dns) {
                for check in &mut checks {
                    check.status = ProbeStatus::Timeout;
                    check.addresses.clear();
                }
            }
            checks
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
            let mut checks = vec![http(MICROSOFT_URL), http(GOOGLE_URL)];
            if matches!(self.scenario, Scenario::Dns | Scenario::External) {
                for check in &mut checks {
                    check.status = ProbeStatus::Timeout;
                    check.status_code = None;
                    check.body_matches = None;
                }
            }
            checks
        }
    }

    struct PanickingCollector;

    impl NetworkCollector for PanickingCollector {
        fn collector_name(&self) -> &'static str {
            "panicking"
        }
        fn collect_inventory(&self) -> NetworkInventory {
            panic!("collector panic")
        }
        fn check_gateway(&self, _: Option<&RouteSnapshot>) -> GatewayCheck {
            unreachable!()
        }
        fn check_dns(&self) -> Vec<DnsCheck> {
            unreachable!()
        }
        fn check_http_endpoint(&self, _: &str) -> HttpCheck {
            unreachable!()
        }
        fn check_http(&self) -> Vec<HttpCheck> {
            unreachable!()
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

    fn run_requests(collector: RecordingCollector, waits: usize) -> Vec<(ProbeRequest, Duration)> {
        let latest = Arc::new(RwLock::new(NetworkDiagnosticStatus::starting()));
        let mut requests = vec![];
        run_worker_observed(
            coordinator(collector),
            ScriptedWait::elapsed(waits),
            latest,
            |request, elapsed| requests.push((request.clone(), elapsed)),
        );
        requests
    }

    #[test]
    fn startup_is_full_then_external_targets_alternate_every_two_baselines() {
        let requests = run_requests(RecordingCollector::normal(Arc::new(Mutex::new(vec![]))), 5);
        assert_eq!(
            requests
                .into_iter()
                .map(|(request, _)| request)
                .collect::<Vec<_>>(),
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
        let requests = run_requests(
            RecordingCollector::unknown_failure(Arc::new(Mutex::new(vec![]))),
            4,
        );
        let focused_at: Vec<_> = requests
            .into_iter()
            .filter_map(|(request, elapsed)| {
                (request == ProbeRequest::Focused).then_some(elapsed.as_secs())
            })
            .collect();
        // The startup full is the first assessment; focused runs at t=0 and t=30.
        assert_eq!(focused_at, vec![0, 30]);
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
        for scenario in [Scenario::Dns, Scenario::External, Scenario::Unknown] {
            let focused_at: Vec<_> = run_requests(RecordingCollector::scenario(scenario), 4)
                .into_iter()
                .filter_map(|(request, elapsed)| {
                    (request == ProbeRequest::Focused).then_some(elapsed.as_secs())
                })
                .collect();
            assert_eq!(focused_at, vec![0, 30]);
        }
        for scenario in [Scenario::Local, Scenario::Gateway] {
            let focused_at: Vec<_> = run_requests(RecordingCollector::scenario(scenario), 4)
                .into_iter()
                .filter_map(|(request, elapsed)| {
                    (request == ProbeRequest::Focused).then_some(elapsed.as_secs())
                })
                .collect();
            assert_eq!(focused_at, vec![0]);
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
    fn blocking_probe_does_not_create_catch_up_waits_or_overlap() {
        let active = Arc::new(AtomicUsize::new(0));
        let maximum = Arc::new(AtomicUsize::new(0));
        let barrier = Arc::new(Barrier::new(2));
        let collector = RecordingCollector {
            active: Some(active),
            maximum_active: Some(maximum.clone()),
            first_call_barrier: Some(barrier.clone()),
            collection_calls: Some(Arc::new(AtomicUsize::new(0))),
            ..RecordingCollector::normal(Arc::new(Mutex::new(vec![])))
        };
        let wait = ScriptedWait::elapsed(3);
        let wait_log = wait.wait_log.clone();
        let fake_clock = wait.elapsed.clone();
        let observed = Arc::new(Mutex::new(vec![]));
        let worker_observed = observed.clone();
        let collector = RecordingCollector {
            fake_clock: Some(fake_clock),
            slow_by: Duration::from_secs(25),
            ..collector
        };
        let latest = Arc::new(RwLock::new(NetworkDiagnosticStatus::starting()));
        let worker = thread::spawn(move || {
            run_worker_observed(coordinator(collector), wait, latest, |request, elapsed| {
                worker_observed
                    .lock()
                    .unwrap()
                    .push((request.clone(), elapsed))
            })
        });

        barrier.wait();
        worker.join().unwrap();

        assert!(wait_log
            .lock()
            .unwrap()
            .iter()
            .all(|duration| *duration == BASELINE_INTERVAL));
        let first_baseline_at = observed
            .lock()
            .unwrap()
            .iter()
            .find_map(|(request, elapsed)| {
                matches!(request, ProbeRequest::Baseline(_)).then_some(*elapsed)
            })
            .unwrap();
        assert_eq!(first_baseline_at, Duration::from_secs(35));
        assert_eq!(maximum.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn automatic_http_requests_stay_within_hourly_limits() {
        let normal = run_requests(
            RecordingCollector::normal(Arc::new(Mutex::new(vec![]))),
            360,
        );
        let persistent = run_requests(
            RecordingCollector::unknown_failure(Arc::new(Mutex::new(vec![]))),
            360,
        );
        let normal_http = normal
            .iter()
            .filter(|(request, _)| matches!(request, ProbeRequest::Baseline(Some(_))))
            .count();
        let persistent_http: usize = persistent
            .iter()
            .map(|(request, elapsed)| match request {
                ProbeRequest::Baseline(Some(_)) => 1,
                ProbeRequest::Focused if !elapsed.is_zero() => 2,
                _ => 0,
            })
            .sum();

        // Startup full and its immediate focused refinement are outside steady state.
        assert_eq!(normal_http, 180);
        assert_eq!(persistent_http, 420);
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
    fn poisoned_status_storage_is_recovered_as_runtime_error() {
        let latest = Arc::new(RwLock::new(NetworkDiagnosticStatus::starting()));
        let poison = latest.clone();
        let _ = thread::spawn(move || {
            let _guard = poison.write().unwrap();
            panic!("poison status lock");
        })
        .join();
        run_worker(
            coordinator(RecordingCollector::normal(Arc::new(Mutex::new(vec![])))),
            ScriptedWait::elapsed(5),
            latest.clone(),
        );
        let runtime = NetworkDiagnosticsRuntime {
            latest,
            stop: StopHandle::inactive(),
            worker: None,
        };
        let status = runtime.status().unwrap();
        assert_eq!(status.availability, RuntimeAvailability::Error);
        assert_eq!(status.error.as_ref().unwrap().stage, "runtime");
        assert_eq!(status.error.as_ref().unwrap().code, "runtime_failed");
        assert_eq!(
            status.error.as_ref().unwrap().message,
            "network diagnostic status lock failed"
        );
        assert_eq!(status.error.as_ref().unwrap().native_code, None);
    }

    #[test]
    fn direct_poisoned_status_read_returns_runtime_error() {
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

        let status = runtime.status().unwrap();
        assert_eq!(status.availability, RuntimeAvailability::Error);
        assert_eq!(status.error.as_ref().unwrap().stage, "runtime");
        assert_eq!(status.error.as_ref().unwrap().code, "runtime_failed");
        assert_eq!(
            status.error.as_ref().unwrap().message,
            "network diagnostic status lock failed"
        );
    }

    #[test]
    fn worker_panic_is_visible_as_runtime_error_before_drop() {
        let coordinator = NetworkProbeCoordinator::from_service_for_test(NetworkProbeService::new(
            Box::new(PanickingCollector),
        ));
        let mut runtime =
            NetworkDiagnosticsRuntime::start_for_test(coordinator, ScriptedWait::elapsed(1));
        runtime.worker.take().unwrap().join().unwrap();

        let status = runtime.status().unwrap();
        assert_eq!(status.availability, RuntimeAvailability::Error);
        assert_eq!(status.error.as_ref().unwrap().stage, "runtime");
        assert_eq!(status.error.as_ref().unwrap().code, "runtime_failed");
        assert_eq!(
            status.error.as_ref().unwrap().message,
            "network diagnostic worker panicked"
        );
        assert_eq!(status.error.as_ref().unwrap().native_code, None);
    }
}
