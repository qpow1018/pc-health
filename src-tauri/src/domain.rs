use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "status", rename_all = "kebab-case")]
pub enum SensorValue {
    Available { value: f64, unit: String },
    UnsupportedDevice,
    UnsupportedApp,
    Waiting,
    Error { message: String },
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum IndicationLevel {
    Advisory,
    Warning,
    HighLoad,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Indication {
    pub level: IndicationLevel,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SensorReading {
    pub kind: String,
    pub label: String,
    pub value: SensorValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indication: Option<Indication>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DeviceKind {
    Cpu,
    Gpu,
    Memory,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceSnapshot {
    pub kind: DeviceKind,
    pub name: String,
    pub readings: Vec<SensorReading>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SensorSnapshot {
    pub collected_at: String,
    pub devices: Vec<DeviceSnapshot>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParsedTelemetry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_usage: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu_clock_mhz: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_memory_kb: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub free_memory_kb: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SensorDiagnostics {
    pub collected_at: String,
    pub collector: String,
    pub duration_ms: u128,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_payload: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raw_error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parsed_telemetry: Option<ParsedTelemetry>,
    pub snapshot: SensorSnapshot,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn available_sensor_value_serializes_with_status_value_and_unit() {
        let value = SensorValue::Available {
            value: 72.5,
            unit: "C".to_string(),
        };

        assert_eq!(
            serde_json::to_value(value).unwrap(),
            json!({
                "status": "available",
                "value": 72.5,
                "unit": "C"
            })
        );
    }

    #[test]
    fn snapshot_serializes_with_authoritative_wire_names() {
        let snapshot = SensorSnapshot {
            collected_at: "2026-06-15T12:00:00Z".to_string(),
            devices: vec![DeviceSnapshot {
                kind: DeviceKind::Gpu,
                name: "GPU".to_string(),
                readings: vec![SensorReading {
                    kind: "temperature".to_string(),
                    label: "온도".to_string(),
                    value: SensorValue::UnsupportedApp,
                    indication: Some(Indication {
                        level: IndicationLevel::HighLoad,
                        message: "높은 부하".to_string(),
                    }),
                }],
            }],
        };

        assert_eq!(
            serde_json::to_value(snapshot).unwrap(),
            json!({
                "collectedAt": "2026-06-15T12:00:00Z",
                "devices": [{
                    "kind": "gpu",
                    "name": "GPU",
                    "readings": [{
                        "kind": "temperature",
                        "label": "온도",
                        "value": { "status": "unsupported-app" },
                        "indication": {
                            "level": "high-load",
                            "message": "높은 부하"
                        }
                    }]
                }]
            })
        );
    }

    #[test]
    fn diagnostics_serializes_with_camel_case_fields() {
        let diagnostics = SensorDiagnostics {
            collected_at: "2026-06-17T12:00:00Z".to_string(),
            collector: "windows-powershell".to_string(),
            duration_ms: 12,
            raw_payload: Some(r#"{"CpuUsage":37}"#.to_string()),
            raw_error: None,
            parsed_telemetry: Some(ParsedTelemetry {
                cpu_name: Some("AMD Ryzen".to_string()),
                cpu_usage: Some(37.0),
                cpu_clock_mhz: None,
                total_memory_kb: None,
                free_memory_kb: None,
            }),
            snapshot: SensorSnapshot {
                collected_at: "2026-06-17T12:00:00Z".to_string(),
                devices: vec![],
            },
        };

        assert_eq!(
            serde_json::to_value(diagnostics).unwrap(),
            json!({
                "collectedAt": "2026-06-17T12:00:00Z",
                "collector": "windows-powershell",
                "durationMs": 12,
                "rawPayload": "{\"CpuUsage\":37}",
                "parsedTelemetry": {
                    "cpuName": "AMD Ryzen",
                    "cpuUsage": 37.0
                },
                "snapshot": {
                    "collectedAt": "2026-06-17T12:00:00Z",
                    "devices": []
                }
            })
        );
    }
}
