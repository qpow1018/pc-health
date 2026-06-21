#![cfg_attr(not(target_os = "windows"), allow(dead_code))]

use serde::{Deserialize, Serialize};

#[cfg(target_os = "windows")]
use std::{
    collections::VecDeque,
    env,
    io::{BufRead, BufReader, Write},
    os::windows::process::CommandExt,
    path::PathBuf,
    process::{Child, ChildStdin, Command, Stdio},
    sync::{
        mpsc::{self, Receiver, RecvTimeoutError},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Threading::CREATE_NO_WINDOW;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SensorHelperRequest<'a> {
    id: u64,
    command: &'a str,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SensorHelperSensor {
    pub name: String,
    pub sensor_type: String,
    pub identifier: String,
    pub parent_identifier: String,
    pub unit: Option<String>,
    pub value: Option<f64>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SensorHelperHardware {
    pub name: String,
    pub hardware_type: String,
    pub identifier: String,
    pub parent_identifier: Option<String>,
    pub update_error: Option<String>,
    pub sensors: Vec<SensorHelperSensor>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SensorHelperResponse {
    pub id: Option<u64>,
    pub ok: bool,
    pub helper_pid: u32,
    pub sample_index: u64,
    pub command: String,
    pub hardware: Vec<SensorHelperHardware>,
    pub errors: Vec<String>,
}

pub fn sample_request(id: u64) -> Result<String, String> {
    request_line(id, "sample")
}

#[cfg(target_os = "windows")]
fn shutdown_request(id: u64) -> Result<String, String> {
    request_line(id, "shutdown")
}

fn request_line(id: u64, command: &str) -> Result<String, String> {
    serde_json::to_string(&SensorHelperRequest { id, command })
        .map(|line| format!("{line}\n"))
        .map_err(|error| format!("sensor helper 요청을 만들지 못했습니다: {error}"))
}

pub fn parse_response(line: &str, expected_id: u64) -> Result<SensorHelperResponse, String> {
    let response: SensorHelperResponse = serde_json::from_str(line.trim())
        .map_err(|error| format!("sensor helper 응답을 해석하지 못했습니다: {error}"))?;

    if response.id != Some(expected_id) {
        return Err(format!(
            "sensor helper request id가 일치하지 않습니다: expected {expected_id}, got {:?}",
            response.id
        ));
    }

    if !response.ok {
        return Err(if response.errors.is_empty() {
            "sensor helper가 실패 응답을 반환했습니다.".into()
        } else {
            response.errors.join("; ")
        });
    }

    Ok(response)
}

pub fn select_cpu_temperature(response: &SensorHelperResponse) -> Option<f64> {
    response
        .hardware
        .iter()
        .filter(|hardware| hardware.hardware_type.eq_ignore_ascii_case("cpu"))
        .flat_map(|hardware| &hardware.sensors)
        .filter(|sensor| sensor.sensor_type.eq_ignore_ascii_case("temperature"))
        .filter_map(|sensor| {
            let value = sensor.value?;
            (value.is_finite() && value > 0.0 && value <= 130.0).then_some((sensor, value))
        })
        .max_by_key(|(sensor, _)| temperature_priority(&sensor.name))
        .map(|(_, value)| value)
}

fn temperature_priority(name: &str) -> u8 {
    let name = name.to_ascii_lowercase();
    if name.contains("package") || name.contains("tctl") || name.contains("tdie") {
        2
    } else {
        1
    }
}

pub fn retry_once<S, T>(
    state: &mut S,
    mut attempt: impl FnMut(&mut S) -> Result<T, String>,
    mut restart: impl FnMut(&mut S) -> Result<(), String>,
) -> Result<T, String> {
    match attempt(state) {
        Ok(value) => Ok(value),
        Err(_) => {
            restart(state)?;
            attempt(state)
        }
    }
}

#[cfg(target_os = "windows")]
pub struct SensorHelperSample {
    pub response: SensorHelperResponse,
    pub raw_line: String,
}

#[cfg(target_os = "windows")]
#[derive(Default)]
pub struct SensorHelperClient {
    process: Option<HelperProcess>,
    next_id: u64,
}

#[cfg(target_os = "windows")]
impl SensorHelperClient {
    pub fn sample(&mut self) -> Result<SensorHelperSample, String> {
        retry_once(self, SensorHelperClient::sample_once, |client| {
            client.reset();
            Ok(())
        })
    }

    fn sample_once(&mut self) -> Result<SensorHelperSample, String> {
        self.ensure_process()?;
        let id = self.next_request_id();
        let request = sample_request(id)?;
        let process = self
            .process
            .as_mut()
            .ok_or_else(|| "sensor helper process가 없습니다.".to_string())?;
        let raw_line = process.exchange(&request, Duration::from_secs(2))?;
        let response = parse_response(&raw_line, id)?;
        Ok(SensorHelperSample { response, raw_line })
    }

    fn ensure_process(&mut self) -> Result<(), String> {
        if self.process.is_none() {
            self.process = Some(HelperProcess::spawn()?);
        }
        Ok(())
    }

    fn next_request_id(&mut self) -> u64 {
        self.next_id = self.next_id.saturating_add(1);
        self.next_id
    }

    fn reset(&mut self) {
        if let Some(mut process) = self.process.take() {
            process.stop();
        }
    }
}

#[cfg(target_os = "windows")]
impl Drop for SensorHelperClient {
    fn drop(&mut self) {
        if let Some(mut process) = self.process.take() {
            let id = self.next_request_id();
            process.shutdown(id);
        }
    }
}

#[cfg(target_os = "windows")]
struct HelperProcess {
    child: Child,
    stdin: ChildStdin,
    responses: Receiver<Result<String, String>>,
    stderr_tail: Arc<Mutex<VecDeque<String>>>,
}

#[cfg(target_os = "windows")]
impl HelperProcess {
    fn spawn() -> Result<Self, String> {
        let helper_path = find_sensor_helper_executable()
            .ok_or_else(|| "sensor helper 실행 파일을 찾지 못했습니다.".to_string())?;
        let mut child = Command::new(helper_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|error| format!("sensor helper 실행에 실패했습니다: {error}"))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "sensor helper stdin을 열지 못했습니다.".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "sensor helper stdout을 열지 못했습니다.".to_string())?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| "sensor helper stderr를 열지 못했습니다.".to_string())?;

        let (sender, responses) = mpsc::channel();
        thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                let result =
                    line.map_err(|error| format!("sensor helper stdout 읽기 실패: {error}"));
                if sender.send(result).is_err() {
                    break;
                }
            }
        });

        let stderr_tail = Arc::new(Mutex::new(VecDeque::with_capacity(20)));
        let stderr_lines = Arc::clone(&stderr_tail);
        thread::spawn(move || {
            for line in BufReader::new(stderr).lines().map_while(Result::ok) {
                if let Ok(mut lines) = stderr_lines.lock() {
                    if lines.len() == 20 {
                        lines.pop_front();
                    }
                    lines.push_back(line);
                }
            }
        });

        Ok(Self {
            child,
            stdin,
            responses,
            stderr_tail,
        })
    }

    fn exchange(&mut self, request: &str, timeout: Duration) -> Result<String, String> {
        self.stdin
            .write_all(request.as_bytes())
            .and_then(|_| self.stdin.flush())
            .map_err(|error| format!("sensor helper 요청 전송 실패: {error}"))?;

        match self.responses.recv_timeout(timeout) {
            Ok(Ok(line)) => Ok(line),
            Ok(Err(message)) => Err(message),
            Err(RecvTimeoutError::Timeout) => Err(self.with_stderr("sensor helper 응답 시간 초과")),
            Err(RecvTimeoutError::Disconnected) => {
                Err(self.with_stderr("sensor helper stdout 연결 종료"))
            }
        }
    }

    fn shutdown(&mut self, id: u64) {
        if let Ok(request) = shutdown_request(id) {
            let _ = self.exchange(&request, Duration::from_millis(500));
        }
        self.wait_or_kill(Duration::from_millis(500));
    }

    fn stop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }

    fn wait_or_kill(&mut self, timeout: Duration) {
        let started = Instant::now();
        while started.elapsed() < timeout {
            match self.child.try_wait() {
                Ok(Some(_)) => return,
                Ok(None) => thread::sleep(Duration::from_millis(25)),
                Err(_) => break,
            }
        }
        self.stop();
    }

    fn with_stderr(&self, message: &str) -> String {
        let tail = self
            .stderr_tail
            .lock()
            .ok()
            .map(|lines| lines.iter().cloned().collect::<Vec<_>>().join(" | "))
            .unwrap_or_default();
        if tail.is_empty() {
            message.into()
        } else {
            format!("{message}: {tail}")
        }
    }
}

#[cfg(target_os = "windows")]
fn find_sensor_helper_executable() -> Option<PathBuf> {
    if let Ok(path) = env::var("PC_HEALTH_SENSOR_HELPER") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Some(path);
        }
    }

    let mut candidates = Vec::new();
    if let Ok(current_exe) = env::current_exe() {
        if let Some(app_dir) = current_exe.parent() {
            candidates.push(app_dir.join("pc-health-sensor-helper.exe"));
            candidates.push(app_dir.join("pc-health-sensor-helper-x86_64-pc-windows-msvc.exe"));
        }
    }
    if let Ok(current_dir) = env::current_dir() {
        candidates.push(
            current_dir
                .join("src-tauri")
                .join("binaries")
                .join("pc-health-sensor-helper-x86_64-pc-windows-msvc.exe"),
        );
        candidates.push(
            current_dir
                .join("binaries")
                .join("pc-health-sensor-helper-x86_64-pc-windows-msvc.exe"),
        );
    }

    candidates.into_iter().find(|path| path.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    const RAW_RESPONSE: &str = r#"{"id":7,"ok":true,"helperPid":42,"sampleIndex":2,"command":"sample","hardware":[{"name":"CPU","hardwareType":"Cpu","identifier":"/cpu/0","parentIdentifier":null,"updateError":null,"sensors":[{"name":"CPU Package","sensorType":"Temperature","identifier":"/cpu/0/temp/0","parentIdentifier":"/cpu/0","unit":"C","value":64.5},{"name":"Missing","sensorType":"Temperature","identifier":"/cpu/0/temp/1","parentIdentifier":"/cpu/0","unit":"C","value":null}]}],"errors":[]}"#;

    #[test]
    fn parses_raw_cpu_sensor_response_without_changing_null_to_zero() {
        let response = parse_response(RAW_RESPONSE, 7).unwrap();

        assert_eq!(response.helper_pid, 42);
        assert_eq!(response.sample_index, 2);
        assert_eq!(response.hardware[0].sensors[1].value, None);
    }

    #[test]
    fn rejects_mismatched_request_ids() {
        let error = parse_response(RAW_RESPONSE, 8).unwrap_err();

        assert!(error.contains("request id"));
    }

    #[test]
    fn serializes_sample_requests_as_one_line_ndjson() {
        let request = sample_request(7).unwrap();

        assert_eq!(request, "{\"id\":7,\"command\":\"sample\"}\n");
    }

    #[test]
    fn selects_valid_package_temperature() {
        let response = parse_response(RAW_RESPONSE, 7).unwrap();

        assert_eq!(select_cpu_temperature(&response), Some(64.5));
    }

    #[test]
    fn retry_once_restarts_after_one_failure() {
        let mut attempts = 0;
        let mut restarts = 0;

        let result = retry_once(
            &mut (),
            |_| {
                attempts += 1;
                if attempts == 1 {
                    Err("first".to_string())
                } else {
                    Ok(42)
                }
            },
            |_| {
                restarts += 1;
                Ok(())
            },
        );

        assert_eq!(result, Ok(42));
        assert_eq!(attempts, 2);
        assert_eq!(restarts, 1);
    }

    #[test]
    fn retry_once_never_attempts_a_third_request() {
        let mut attempts = 0;

        let result: Result<(), String> = retry_once(
            &mut (),
            |_| {
                attempts += 1;
                Err(format!("failure {attempts}"))
            },
            |_| Ok(()),
        );

        assert_eq!(attempts, 2);
        assert_eq!(result.unwrap_err(), "failure 2");
    }
}
