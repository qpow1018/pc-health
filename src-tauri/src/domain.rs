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
}
