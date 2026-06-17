use std::time::Instant;

use crate::collector::{MockCollector, MockScenario, SensorCollector};
use crate::domain::{
    DeviceKind, DeviceSnapshot, ParsedTelemetry, SensorDiagnostics, SensorReading, SensorSnapshot,
    SensorValue,
};

#[cfg(target_os = "windows")]
use std::{
    mem::{size_of, zeroed},
    ptr::{null, null_mut},
    thread,
    time::Duration,
};

#[cfg(target_os = "windows")]
use windows_sys::Win32::{
    Foundation::{ERROR_SUCCESS, FILETIME},
    System::{
        Registry::{
            RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_LOCAL_MACHINE, KEY_READ,
            REG_DWORD, REG_SZ,
        },
        SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX},
        Threading::GetSystemTimes,
    },
};

#[cfg(target_os = "windows")]
use windows::{
    core::{w, Interface, BSTR, PCWSTR, VARIANT},
    Win32::{
        Foundation::RPC_E_TOO_LATE,
        System::{
            Com::{
                CoCreateInstance, CoInitializeEx, CoInitializeSecurity, CoUninitialize,
                CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, EOAC_NONE, RPC_C_AUTHN_LEVEL_DEFAULT,
                RPC_C_IMP_LEVEL_IMPERSONATE,
            },
            Variant::{VariantClear, VT_BSTR, VT_R4, VT_R8},
            Wmi::{
                IWbemClassObject, IWbemContext, IWbemLocator, WbemLocator, WBEM_FLAG_FORWARD_ONLY,
                WBEM_FLAG_RETURN_IMMEDIATELY, WBEM_INFINITE,
            },
        },
    },
};

#[derive(Default)]
pub struct WindowsCollector {
    mock: MockCollector,
    #[cfg(target_os = "windows")]
    previous_cpu_times: Option<CpuTimes>,
}

#[cfg(target_os = "windows")]
#[derive(Clone, Copy, Debug)]
struct CpuTimes {
    idle: u64,
    kernel: u64,
    user: u64,
}

#[derive(Clone, Debug)]
struct WindowsTelemetry {
    cpu_name: Option<String>,
    cpu_usage: Option<f64>,
    cpu_temperature_celsius: Option<f64>,
    cpu_clock_mhz: Option<f64>,
    total_memory_kb: Option<f64>,
    free_memory_kb: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
struct HardwareMonitorSensor {
    name: String,
    identifier: String,
    sensor_type: String,
    value: Option<f64>,
}

impl WindowsCollector {
    pub fn new() -> Self {
        Self::default()
    }

    fn collect_live(&mut self, collected_at: String) -> SensorSnapshot {
        self.diagnose_live(collected_at).snapshot
    }

    pub fn diagnose_live(&mut self, collected_at: String) -> SensorDiagnostics {
        let started_at = Instant::now();
        let duration = || started_at.elapsed().as_millis();

        match self.collect_native_telemetry() {
            Ok(telemetry) => diagnostics_from_telemetry(collected_at, duration(), telemetry),
            Err(message) => SensorDiagnostics {
                collected_at: collected_at.clone(),
                collector: "windows-native".into(),
                duration_ms: duration(),
                raw_payload: None,
                raw_error: Some(message.clone()),
                parsed_telemetry: None,
                snapshot: error_snapshot(collected_at, message),
            },
        }
    }

    #[cfg(target_os = "windows")]
    fn collect_native_telemetry(&mut self) -> Result<WindowsTelemetry, String> {
        let first_cpu_times = match self.previous_cpu_times {
            Some(times) => times,
            None => {
                let times = read_cpu_times()?;
                thread::sleep(Duration::from_millis(100));
                times
            }
        };
        let current_cpu_times = read_cpu_times()?;
        self.previous_cpu_times = Some(current_cpu_times);

        let memory = read_memory_status()?;

        Ok(WindowsTelemetry {
            cpu_name: Some(read_processor_name().unwrap_or_else(|_| "Windows CPU".into())),
            cpu_usage: calculate_cpu_usage(first_cpu_times, current_cpu_times),
            cpu_temperature_celsius: read_cpu_temperature_celsius().unwrap_or(None),
            cpu_clock_mhz: Some(read_processor_mhz()? as f64),
            total_memory_kb: Some(memory.total_kb),
            free_memory_kb: Some(memory.free_kb),
        })
    }

    #[cfg(not(target_os = "windows"))]
    fn collect_native_telemetry(&mut self) -> Result<WindowsTelemetry, String> {
        Err("Windows native sensor collection requires Windows.".into())
    }
}

impl From<WindowsTelemetry> for ParsedTelemetry {
    fn from(telemetry: WindowsTelemetry) -> Self {
        Self {
            cpu_name: telemetry.cpu_name,
            cpu_usage: telemetry.cpu_usage,
            cpu_temperature_celsius: telemetry.cpu_temperature_celsius,
            cpu_clock_mhz: telemetry.cpu_clock_mhz,
            total_memory_kb: telemetry.total_memory_kb,
            free_memory_kb: telemetry.free_memory_kb,
        }
    }
}

fn diagnostics_from_telemetry(
    collected_at: String,
    duration_ms: u128,
    telemetry: WindowsTelemetry,
) -> SensorDiagnostics {
    match snapshot_from_telemetry(collected_at.clone(), telemetry.clone()) {
        Ok(snapshot) => SensorDiagnostics {
            collected_at,
            collector: "windows-native".into(),
            duration_ms,
            raw_payload: None,
            raw_error: None,
            parsed_telemetry: Some(telemetry.into()),
            snapshot,
        },
        Err(message) => SensorDiagnostics {
            collected_at: collected_at.clone(),
            collector: "windows-native".into(),
            duration_ms,
            raw_payload: None,
            raw_error: Some(message.clone()),
            parsed_telemetry: Some(telemetry.into()),
            snapshot: error_snapshot(collected_at, message),
        },
    }
}

#[cfg(target_os = "windows")]
#[derive(Clone, Copy, Debug)]
struct MemoryStatus {
    total_kb: f64,
    free_kb: f64,
}

#[cfg(target_os = "windows")]
fn read_cpu_times() -> Result<CpuTimes, String> {
    unsafe {
        let mut idle = zeroed::<FILETIME>();
        let mut kernel = zeroed::<FILETIME>();
        let mut user = zeroed::<FILETIME>();

        if GetSystemTimes(&mut idle, &mut kernel, &mut user) == 0 {
            return Err("CPU 사용률 시간을 읽지 못했습니다.".into());
        }

        Ok(CpuTimes {
            idle: filetime_to_u64(idle),
            kernel: filetime_to_u64(kernel),
            user: filetime_to_u64(user),
        })
    }
}

#[cfg(target_os = "windows")]
fn calculate_cpu_usage(previous: CpuTimes, current: CpuTimes) -> Option<f64> {
    let previous_total = previous.kernel.saturating_add(previous.user);
    let current_total = current.kernel.saturating_add(current.user);
    let total_delta = current_total.saturating_sub(previous_total);
    let idle_delta = current.idle.saturating_sub(previous.idle);

    if total_delta == 0 {
        None
    } else {
        Some(((total_delta.saturating_sub(idle_delta)) as f64 / total_delta as f64) * 100.0)
    }
}

#[cfg(target_os = "windows")]
fn filetime_to_u64(filetime: FILETIME) -> u64 {
    ((filetime.dwHighDateTime as u64) << 32) | filetime.dwLowDateTime as u64
}

#[cfg(target_os = "windows")]
fn read_memory_status() -> Result<MemoryStatus, String> {
    unsafe {
        let mut status = zeroed::<MEMORYSTATUSEX>();
        status.dwLength = size_of::<MEMORYSTATUSEX>() as u32;

        if GlobalMemoryStatusEx(&mut status) == 0 {
            return Err("메모리 상태를 읽지 못했습니다.".into());
        }

        Ok(MemoryStatus {
            total_kb: status.ullTotalPhys as f64 / 1024.0,
            free_kb: status.ullAvailPhys as f64 / 1024.0,
        })
    }
}

#[cfg(target_os = "windows")]
fn read_processor_name() -> Result<String, String> {
    read_registry_string(
        "HARDWARE\\DESCRIPTION\\System\\CentralProcessor\\0",
        "ProcessorNameString",
    )
}

#[cfg(target_os = "windows")]
fn read_processor_mhz() -> Result<u32, String> {
    read_registry_dword("HARDWARE\\DESCRIPTION\\System\\CentralProcessor\\0", "~MHz")
}

#[cfg(target_os = "windows")]
fn read_registry_string(subkey: &str, value_name: &str) -> Result<String, String> {
    unsafe {
        let key = open_registry_key(subkey)?;
        let value_name = to_wide(value_name);
        let mut value_type = 0;
        let mut buffer = [0u16; 256];
        let mut byte_len = (buffer.len() * size_of::<u16>()) as u32;
        let result = RegQueryValueExW(
            key,
            value_name.as_ptr(),
            null(),
            &mut value_type,
            buffer.as_mut_ptr().cast(),
            &mut byte_len,
        );
        RegCloseKey(key);

        if result != ERROR_SUCCESS || value_type != REG_SZ {
            return Err("CPU 이름 레지스트리 값을 읽지 못했습니다.".into());
        }

        let len = buffer
            .iter()
            .position(|item| *item == 0)
            .unwrap_or(buffer.len());
        Ok(String::from_utf16_lossy(&buffer[..len]).trim().to_string())
    }
}

#[cfg(target_os = "windows")]
fn read_registry_dword(subkey: &str, value_name: &str) -> Result<u32, String> {
    unsafe {
        let key = open_registry_key(subkey)?;
        let value_name = to_wide(value_name);
        let mut value_type = 0;
        let mut value = 0u32;
        let mut byte_len = size_of::<u32>() as u32;
        let result = RegQueryValueExW(
            key,
            value_name.as_ptr(),
            null(),
            &mut value_type,
            (&mut value as *mut u32).cast(),
            &mut byte_len,
        );
        RegCloseKey(key);

        if result != ERROR_SUCCESS || value_type != REG_DWORD {
            return Err("CPU 클럭 레지스트리 값을 읽지 못했습니다.".into());
        }

        Ok(value)
    }
}

#[cfg(target_os = "windows")]
fn open_registry_key(subkey: &str) -> Result<HKEY, String> {
    unsafe {
        let subkey = to_wide(subkey);
        let mut key = null_mut();
        let result = RegOpenKeyExW(HKEY_LOCAL_MACHINE, subkey.as_ptr(), 0, KEY_READ, &mut key);
        if result != ERROR_SUCCESS {
            Err("CPU 레지스트리 키를 열지 못했습니다.".into())
        } else {
            Ok(key)
        }
    }
}

#[cfg(target_os = "windows")]
fn to_wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}

#[cfg(target_os = "windows")]
fn read_cpu_temperature_celsius() -> Result<Option<f64>, String> {
    let sensors = read_hardware_monitor_sensors()?;
    Ok(select_cpu_temperature_celsius(&sensors))
}

#[cfg(target_os = "windows")]
fn read_hardware_monitor_sensors() -> Result<Vec<HardwareMonitorSensor>, String> {
    let _com = ComApartment::new()?;
    let mut sensors = Vec::new();

    for namespace in ["ROOT\\LibreHardwareMonitor", "ROOT\\OpenHardwareMonitor"] {
        if let Ok(mut namespace_sensors) = read_hardware_monitor_namespace(namespace) {
            sensors.append(&mut namespace_sensors);
        }
    }

    Ok(sensors)
}

#[cfg(target_os = "windows")]
struct ComApartment;

#[cfg(target_os = "windows")]
impl ComApartment {
    fn new() -> Result<Self, String> {
        unsafe {
            let hr = CoInitializeEx(None, COINIT_MULTITHREADED);
            if hr.is_err() {
                return Err(format!("COM 초기화에 실패했습니다: {hr:?}"));
            }

            match CoInitializeSecurity(
                None,
                -1,
                None,
                None,
                RPC_C_AUTHN_LEVEL_DEFAULT,
                RPC_C_IMP_LEVEL_IMPERSONATE,
                None,
                EOAC_NONE,
                None,
            ) {
                Ok(()) => {}
                Err(error) if error.code() == RPC_E_TOO_LATE => {}
                Err(error) => return Err(format!("COM 보안 초기화에 실패했습니다: {error}")),
            }
        }

        Ok(Self)
    }
}

#[cfg(target_os = "windows")]
impl Drop for ComApartment {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}

#[cfg(target_os = "windows")]
fn read_hardware_monitor_namespace(namespace: &str) -> Result<Vec<HardwareMonitorSensor>, String> {
    unsafe {
        let locator: IWbemLocator = CoCreateInstance(&WbemLocator, None, CLSCTX_INPROC_SERVER)
            .map_err(|error| format!("WMI locator를 만들지 못했습니다: {error}"))?;
        let services = locator
            .ConnectServer(
                &BSTR::from(namespace),
                &BSTR::new(),
                &BSTR::new(),
                &BSTR::new(),
                0,
                &BSTR::new(),
                None::<&IWbemContext>,
            )
            .map_err(|error| format!("{namespace} namespace에 연결하지 못했습니다: {error}"))?;

        let flags = WBEM_FLAG_FORWARD_ONLY | WBEM_FLAG_RETURN_IMMEDIATELY;
        let enumerator = services
            .ExecQuery(
                &BSTR::from("WQL"),
                &BSTR::from("SELECT Name, Identifier, SensorType, Value FROM Sensor"),
                flags,
                None::<&IWbemContext>,
            )
            .map_err(|error| format!("{namespace} 센서 쿼리에 실패했습니다: {error}"))?;

        let mut sensors = Vec::new();
        loop {
            let mut returned = 0;
            let mut objects = [None::<IWbemClassObject>];
            enumerator
                .Next(WBEM_INFINITE, &mut objects, &mut returned)
                .map_err(|error| format!("{namespace} 센서 열거에 실패했습니다: {error}"))?;

            if returned == 0 {
                break;
            }

            if let Some(object) = objects[0].take() {
                if let Some(sensor) = sensor_from_wmi_object(&object) {
                    sensors.push(sensor);
                }
            }
        }

        Ok(sensors)
    }
}

#[cfg(target_os = "windows")]
fn sensor_from_wmi_object(object: &IWbemClassObject) -> Option<HardwareMonitorSensor> {
    Some(HardwareMonitorSensor {
        name: get_wmi_string(object, w!("Name"))?,
        identifier: get_wmi_string(object, w!("Identifier"))?,
        sensor_type: get_wmi_string(object, w!("SensorType"))?,
        value: get_wmi_f64(object, w!("Value")),
    })
}

#[cfg(target_os = "windows")]
fn get_wmi_string(object: &IWbemClassObject, name: PCWSTR) -> Option<String> {
    let mut variant = VARIANT::default();
    let value = unsafe {
        object.Get(name, 0, &mut variant, None, None).ok()?;
        let value = variant_string(&variant);
        let _ = VariantClear(&mut variant);
        value
    };

    value
}

#[cfg(target_os = "windows")]
fn get_wmi_f64(object: &IWbemClassObject, name: PCWSTR) -> Option<f64> {
    let mut variant = VARIANT::default();
    let value = unsafe {
        object.Get(name, 0, &mut variant, None, None).ok()?;
        let value = variant_f64(&variant);
        let _ = VariantClear(&mut variant);
        value
    };

    value
}

fn select_cpu_temperature_celsius(sensors: &[HardwareMonitorSensor]) -> Option<f64> {
    let mut candidates = sensors
        .iter()
        .filter_map(|sensor| cpu_temperature_candidate(sensor).map(|value| (sensor, value)))
        .collect::<Vec<_>>();

    candidates.sort_by(|(left_sensor, left_value), (right_sensor, right_value)| {
        sensor_temperature_priority(right_sensor)
            .cmp(&sensor_temperature_priority(left_sensor))
            .then_with(|| right_value.total_cmp(left_value))
    });

    candidates.first().map(|(_, value)| *value)
}

fn cpu_temperature_candidate(sensor: &HardwareMonitorSensor) -> Option<f64> {
    if !sensor.sensor_type.eq_ignore_ascii_case("Temperature") {
        return None;
    }

    let identifier = sensor.identifier.to_ascii_lowercase();
    if !(identifier.contains("/amdcpu/")
        || identifier.contains("/intelcpu/")
        || identifier.contains("/cpu/"))
    {
        return None;
    }

    let value = sensor.value?;
    if value.is_finite() && (0.0..=130.0).contains(&value) {
        Some(value)
    } else {
        None
    }
}

fn sensor_temperature_priority(sensor: &HardwareMonitorSensor) -> u8 {
    let name = sensor.name.to_ascii_lowercase();

    if name.contains("package") || name.contains("tctl") || name.contains("tdie") {
        2
    } else {
        1
    }
}

#[cfg(target_os = "windows")]
fn variant_string(variant: &VARIANT) -> Option<String> {
    unsafe {
        let inner = &variant.Anonymous.Anonymous;
        if inner.vt != VT_BSTR {
            return None;
        }

        Some(inner.Anonymous.bstrVal.to_string())
    }
}

#[cfg(target_os = "windows")]
fn variant_f64(variant: &VARIANT) -> Option<f64> {
    unsafe {
        let inner = &variant.Anonymous.Anonymous;
        if inner.vt == VT_R8 {
            Some(inner.Anonymous.dblVal)
        } else if inner.vt == VT_R4 {
            Some(inner.Anonymous.fltVal as f64)
        } else {
            None
        }
    }
}

fn available(kind: &str, label: &str, value: f64, unit: &str) -> SensorReading {
    SensorReading {
        kind: kind.into(),
        label: label.into(),
        value: SensorValue::Available {
            value,
            unit: unit.into(),
        },
        indication: None,
    }
}

fn unsupported_app(kind: &str, label: &str) -> SensorReading {
    SensorReading {
        kind: kind.into(),
        label: label.into(),
        value: SensorValue::UnsupportedApp,
        indication: None,
    }
}

fn optional_available(kind: &str, label: &str, value: Option<f64>, unit: &str) -> SensorReading {
    match value {
        Some(value) => available(kind, label, value, unit),
        None => unsupported_app(kind, label),
    }
}

fn error(kind: &str, label: &str, message: &str) -> SensorReading {
    SensorReading {
        kind: kind.into(),
        label: label.into(),
        value: SensorValue::Error {
            message: message.into(),
        },
        indication: None,
    }
}

fn snapshot_from_telemetry(
    collected_at: String,
    telemetry: WindowsTelemetry,
) -> Result<SensorSnapshot, String> {
    let cpu_name = telemetry
        .cpu_name
        .map(|name| name.trim().to_string())
        .filter(|name| !name.is_empty())
        .ok_or_else(|| "CPU 이름을 읽지 못했습니다.".to_string())?;
    let cpu_usage = telemetry
        .cpu_usage
        .ok_or_else(|| "CPU 사용률을 읽지 못했습니다.".to_string())?;
    let cpu_clock_mhz = telemetry
        .cpu_clock_mhz
        .ok_or_else(|| "CPU 클럭을 읽지 못했습니다.".to_string())?;
    let total_memory_kb = telemetry
        .total_memory_kb
        .ok_or_else(|| "전체 메모리를 읽지 못했습니다.".to_string())?;
    let free_memory_kb = telemetry
        .free_memory_kb
        .ok_or_else(|| "사용 가능 메모리를 읽지 못했습니다.".to_string())?;
    let used_memory_kb = total_memory_kb - free_memory_kb;
    let memory_usage = used_memory_kb / total_memory_kb * 100.0;
    let memory_used_gb = used_memory_kb / 1024.0 / 1024.0;

    Ok(SensorSnapshot {
        collected_at,
        devices: vec![
            DeviceSnapshot {
                kind: DeviceKind::Cpu,
                name: cpu_name,
                readings: vec![
                    available("cpu_usage", "사용률", cpu_usage, "%"),
                    optional_available(
                        "cpu_temperature",
                        "온도",
                        telemetry.cpu_temperature_celsius,
                        "C",
                    ),
                    available("cpu_clock", "클럭", cpu_clock_mhz / 1000.0, "GHz"),
                    unsupported_app("cpu_power", "전력"),
                ],
            },
            DeviceSnapshot {
                kind: DeviceKind::Gpu,
                name: "NVIDIA GPU".into(),
                readings: vec![
                    unsupported_app("gpu_usage", "사용률"),
                    unsupported_app("gpu_temperature", "온도"),
                    unsupported_app("gpu_clock", "클럭"),
                    unsupported_app("gpu_power", "전력"),
                    unsupported_app("gpu_fan", "팬"),
                    unsupported_app("gpu_vram", "VRAM"),
                ],
            },
            DeviceSnapshot {
                kind: DeviceKind::Memory,
                name: "System Memory".into(),
                readings: vec![
                    available("memory_usage", "사용률", memory_usage, "%"),
                    available("memory_used", "사용 중", memory_used_gb, "GB"),
                ],
            },
        ],
    })
}

fn error_snapshot(collected_at: String, message: String) -> SensorSnapshot {
    SensorSnapshot {
        collected_at,
        devices: vec![
            DeviceSnapshot {
                kind: DeviceKind::Cpu,
                name: "Windows CPU".into(),
                readings: vec![
                    error("cpu_usage", "사용률", &message),
                    unsupported_app("cpu_temperature", "온도"),
                    error("cpu_clock", "클럭", &message),
                    unsupported_app("cpu_power", "전력"),
                ],
            },
            DeviceSnapshot {
                kind: DeviceKind::Gpu,
                name: "NVIDIA GPU".into(),
                readings: vec![
                    unsupported_app("gpu_usage", "사용률"),
                    unsupported_app("gpu_temperature", "온도"),
                    unsupported_app("gpu_clock", "클럭"),
                    unsupported_app("gpu_power", "전력"),
                    unsupported_app("gpu_fan", "팬"),
                    unsupported_app("gpu_vram", "VRAM"),
                ],
            },
            DeviceSnapshot {
                kind: DeviceKind::Memory,
                name: "System Memory".into(),
                readings: vec![
                    error("memory_usage", "사용률", &message),
                    error("memory_used", "사용 중", &message),
                ],
            },
        ],
    }
}

fn non_empty_or_default(message: String) -> String {
    if message.is_empty() {
        "Windows 센서 데이터를 읽지 못했습니다.".into()
    } else {
        message
    }
}

impl SensorCollector for WindowsCollector {
    fn collect(&mut self, scenario: MockScenario, collected_at: String) -> SensorSnapshot {
        match scenario {
            MockScenario::Live => self.collect_live(collected_at),
            scenario => self.mock.collect(scenario, collected_at),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn available_value(value: &SensorValue) -> f64 {
        match value {
            SensorValue::Available { value, .. } => *value,
            other => panic!("expected available value, got {other:?}"),
        }
    }

    #[test]
    fn windows_telemetry_maps_to_cpu_memory_snapshot() {
        let snapshot = snapshot_from_telemetry(
            "2026-06-17T12:00:00Z".into(),
            WindowsTelemetry {
                cpu_name: Some("AMD Ryzen 7   ".into()),
                cpu_usage: Some(37.0),
                cpu_temperature_celsius: Some(64.5),
                cpu_clock_mhz: Some(4200.0),
                total_memory_kb: Some(33554432.0),
                free_memory_kb: Some(16777216.0),
            },
        )
        .unwrap();

        assert_eq!(snapshot.collected_at, "2026-06-17T12:00:00Z");
        assert_eq!(snapshot.devices[0].kind, DeviceKind::Cpu);
        assert_eq!(snapshot.devices[0].name, "AMD Ryzen 7");
        assert_eq!(
            available_value(&snapshot.devices[0].readings[0].value),
            37.0
        );
        assert_eq!(available_value(&snapshot.devices[0].readings[2].value), 4.2);
        assert_eq!(
            available_value(&snapshot.devices[0].readings[1].value),
            64.5
        );
        assert_eq!(
            snapshot.devices[1].readings[0].value,
            SensorValue::UnsupportedApp
        );
        assert_eq!(
            available_value(&snapshot.devices[2].readings[0].value),
            50.0
        );
        assert_eq!(
            available_value(&snapshot.devices[2].readings[1].value),
            16.0
        );
    }

    #[test]
    fn native_diagnostics_have_no_raw_payload() {
        let diagnostics = diagnostics_from_telemetry(
            "2026-06-17T12:00:00Z".into(),
            8,
            WindowsTelemetry {
                cpu_name: Some("AMD Ryzen 7".into()),
                cpu_usage: Some(37.0),
                cpu_temperature_celsius: None,
                cpu_clock_mhz: Some(4200.0),
                total_memory_kb: Some(33554432.0),
                free_memory_kb: Some(16777216.0),
            },
        );

        assert_eq!(diagnostics.collector, "windows-native");
        assert_eq!(diagnostics.duration_ms, 8);
        assert!(diagnostics.raw_payload.is_none());
        assert!(diagnostics.raw_error.is_none());
        assert_eq!(
            diagnostics.parsed_telemetry.as_ref().unwrap().cpu_usage,
            Some(37.0)
        );
    }

    #[test]
    fn missing_cpu_temperature_stays_explicitly_unsupported() {
        let snapshot = snapshot_from_telemetry(
            "2026-06-17T12:00:00Z".into(),
            WindowsTelemetry {
                cpu_name: Some("AMD Ryzen 7".into()),
                cpu_usage: Some(37.0),
                cpu_temperature_celsius: None,
                cpu_clock_mhz: Some(4200.0),
                total_memory_kb: Some(33554432.0),
                free_memory_kb: Some(16777216.0),
            },
        )
        .unwrap();

        assert_eq!(
            snapshot.devices[0].readings[1].value,
            SensorValue::UnsupportedApp
        );
    }

    #[test]
    fn cpu_temperature_prefers_package_sensor() {
        let sensors = vec![
            HardwareMonitorSensor {
                name: "Core #1".into(),
                identifier: "/amdcpu/0/temperature/1".into(),
                sensor_type: "Temperature".into(),
                value: Some(62.0),
            },
            HardwareMonitorSensor {
                name: "CPU Package".into(),
                identifier: "/amdcpu/0/temperature/2".into(),
                sensor_type: "Temperature".into(),
                value: Some(66.0),
            },
            HardwareMonitorSensor {
                name: "GPU Core".into(),
                identifier: "/gpu-nvidia/0/temperature/0".into(),
                sensor_type: "Temperature".into(),
                value: Some(54.0),
            },
        ];

        assert_eq!(select_cpu_temperature_celsius(&sensors), Some(66.0));
    }
}
