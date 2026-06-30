use super::domain::{
    DiagnosticArea, DiagnosticEvidence, EvidenceSource, EvidenceStatus, NetworkIncident,
    NetworkIncidentEvidence, NetworkIncidentStatus,
};
use chrono::{DateTime, Duration, Utc};
use rusqlite::{params, Connection};
use std::{
    path::Path,
    sync::{Arc, Mutex},
};

const REPRESENTATIVE_EVIDENCE_LIMIT: usize = 4;
const REPRESENTATIVE_EVIDENCE_LIMIT_SQL: i64 = 4;

#[derive(Clone)]
#[cfg_attr(not(test), allow(dead_code))]
pub struct NetworkIncidentStore {
    connection: Arc<Mutex<Connection>>,
}

#[cfg_attr(not(test), allow(dead_code))]
impl NetworkIncidentStore {
    #[allow(dead_code)]
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        let connection = Connection::open(path).map_err(|error| error.to_string())?;
        let store = Self {
            connection: Arc::new(Mutex::new(connection)),
        };
        store.migrate()?;
        store.prune_probe_observations(Utc::now())?;
        Ok(store)
    }

    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self, String> {
        let store = Self {
            connection: Arc::new(Mutex::new(
                Connection::open_in_memory().map_err(|error| error.to_string())?,
            )),
        };
        store.migrate()?;
        Ok(store)
    }

    fn migrate(&self) -> Result<(), String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "incident store lock failed".to_string())?;
        connection
            .execute_batch(
                "
                PRAGMA foreign_keys = ON;
                CREATE TABLE IF NOT EXISTS network_incidents (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    status TEXT NOT NULL CHECK(status IN ('ongoing', 'recovering', 'resolved')),
                    area TEXT NOT NULL,
                    started_at TEXT NOT NULL,
                    last_observed_at TEXT NOT NULL,
                    resolved_at TEXT NULL,
                    summary TEXT NOT NULL
                );
                CREATE TABLE IF NOT EXISTS network_incident_evidence (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    incident_id INTEGER NOT NULL REFERENCES network_incidents(id) ON DELETE CASCADE,
                    source TEXT NOT NULL,
                    status TEXT NOT NULL,
                    checked_at TEXT NULL,
                    duration_ms INTEGER NULL,
                    detail TEXT NULL,
                    observed_at TEXT NOT NULL
                );
                CREATE TABLE IF NOT EXISTS network_probe_observations (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    observed_at TEXT NOT NULL,
                    lifecycle TEXT NULL,
                    suspected_area TEXT NULL,
                    evidence_json TEXT NOT NULL
                );
                ",
            )
            .map_err(|error| error.to_string())
    }

    pub fn create_incident(
        &self,
        area: DiagnosticArea,
        observed_at: &str,
        summary: &str,
        evidence: &[DiagnosticEvidence],
    ) -> Result<i64, String> {
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| "incident store lock failed".to_string())?;
        let transaction = connection
            .transaction()
            .map_err(|error| error.to_string())?;
        transaction
            .execute(
                "INSERT INTO network_incidents (status, area, started_at, last_observed_at, resolved_at, summary)
                 VALUES (?1, ?2, ?3, ?4, NULL, ?5)",
                params![
                    NetworkIncidentStatus::Ongoing.as_str(),
                    area.as_str(),
                    observed_at,
                    observed_at,
                    summary,
                ],
            )
            .map_err(|error| error.to_string())?;
        let id = transaction.last_insert_rowid();
        insert_evidence(&transaction, id, observed_at, evidence)?;
        prune_incident_evidence(&transaction, id)?;
        transaction.commit().map_err(|error| error.to_string())?;
        Ok(id)
    }

    pub fn update_incident(
        &self,
        id: i64,
        status: NetworkIncidentStatus,
        area: DiagnosticArea,
        observed_at: &str,
        resolved_at: Option<&str>,
        summary: &str,
        evidence: &[DiagnosticEvidence],
    ) -> Result<(), String> {
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| "incident store lock failed".to_string())?;
        let transaction = connection
            .transaction()
            .map_err(|error| error.to_string())?;
        transaction
            .execute(
                "UPDATE network_incidents
                 SET status = ?1, area = ?2, last_observed_at = ?3, resolved_at = ?4, summary = ?5
                 WHERE id = ?6",
                params![
                    status.as_str(),
                    area.as_str(),
                    observed_at,
                    resolved_at,
                    summary,
                    id
                ],
            )
            .map_err(|error| error.to_string())?;
        insert_evidence(&transaction, id, observed_at, evidence)?;
        prune_incident_evidence(&transaction, id)?;
        transaction.commit().map_err(|error| error.to_string())
    }

    pub fn recent_incidents(&self, limit: usize) -> Result<Vec<NetworkIncident>, String> {
        let limit = i64::try_from(limit).map_err(|_| "incident limit is too large".to_string())?;
        let connection = self
            .connection
            .lock()
            .map_err(|_| "incident store lock failed".to_string())?;
        let mut statement = connection
            .prepare(
                "SELECT id, status, area, started_at, last_observed_at, resolved_at, summary
                 FROM network_incidents
                 ORDER BY started_at DESC, id DESC
                 LIMIT ?1",
            )
            .map_err(|error| error.to_string())?;
        let rows = statement
            .query_map(params![limit], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, Option<String>>(5)?,
                    row.get::<_, String>(6)?,
                ))
            })
            .map_err(|error| error.to_string())?;

        let mut incidents = Vec::new();
        for row in rows {
            let (id, status, area, started_at, last_observed_at, resolved_at, summary) =
                row.map_err(|error| error.to_string())?;
            incidents.push(NetworkIncident {
                id,
                status: NetworkIncidentStatus::from_str(&status)?,
                area: DiagnosticArea::from_str(&area)?,
                started_at,
                last_observed_at,
                resolved_at,
                summary,
                representative_evidence: incident_evidence(&connection, id)?,
            });
        }
        Ok(incidents)
    }

    pub fn record_probe_observation(
        &self,
        observed_at: &str,
        lifecycle: Option<&str>,
        suspected_area: Option<&str>,
        evidence: &[DiagnosticEvidence],
    ) -> Result<(), String> {
        let evidence_json = serde_json::to_string(evidence).map_err(|error| error.to_string())?;
        let connection = self
            .connection
            .lock()
            .map_err(|_| "incident store lock failed".to_string())?;
        connection
            .execute(
                "INSERT INTO network_probe_observations (observed_at, lifecycle, suspected_area, evidence_json)
                 VALUES (?1, ?2, ?3, ?4)",
                params![observed_at, lifecycle, suspected_area, evidence_json],
            )
            .map_err(|error| error.to_string())?;
        if let Ok(now) = DateTime::parse_from_rfc3339(observed_at) {
            let _ = prune_probe_observations(&connection, now.with_timezone(&Utc));
        }
        Ok(())
    }

    pub fn prune_probe_observations(&self, now: DateTime<Utc>) -> Result<(), String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "incident store lock failed".to_string())?;
        prune_probe_observations(&connection, now)
    }

    #[cfg(test)]
    pub fn probe_observation_count(&self) -> Result<i64, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "incident store lock failed".to_string())?;
        connection
            .query_row(
                "SELECT COUNT(*) FROM network_probe_observations",
                [],
                |row| row.get(0),
            )
            .map_err(|error| error.to_string())
    }

    #[cfg(test)]
    pub fn incident_evidence_count(&self, incident_id: i64) -> Result<i64, String> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| "incident store lock failed".to_string())?;
        connection
            .query_row(
                "SELECT COUNT(*) FROM network_incident_evidence WHERE incident_id = ?1",
                params![incident_id],
                |row| row.get(0),
            )
            .map_err(|error| error.to_string())
    }
}

fn insert_evidence(
    transaction: &rusqlite::Transaction<'_>,
    incident_id: i64,
    observed_at: &str,
    evidence: &[DiagnosticEvidence],
) -> Result<(), String> {
    for item in evidence.iter().take(REPRESENTATIVE_EVIDENCE_LIMIT) {
        let duration_ms = item
            .duration_ms
            .map(i64::try_from)
            .transpose()
            .map_err(|_| "evidence duration is too large".to_string())?;
        transaction
            .execute(
                "INSERT INTO network_incident_evidence
                 (incident_id, source, status, checked_at, duration_ms, detail, observed_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    incident_id,
                    item.source.as_str(),
                    item.status.as_str(),
                    item.checked_at,
                    duration_ms,
                    item.detail,
                    observed_at,
                ],
            )
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn prune_incident_evidence(
    transaction: &rusqlite::Transaction<'_>,
    incident_id: i64,
) -> Result<(), String> {
    transaction
        .execute(
            "DELETE FROM network_incident_evidence
             WHERE incident_id = ?1
             AND id NOT IN (
                 SELECT id FROM network_incident_evidence
                 WHERE incident_id = ?1
                 ORDER BY observed_at DESC, id DESC
                 LIMIT ?2
             )",
            params![incident_id, REPRESENTATIVE_EVIDENCE_LIMIT_SQL],
        )
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[cfg_attr(not(test), allow(dead_code))]
fn incident_evidence(
    connection: &Connection,
    incident_id: i64,
) -> Result<Vec<NetworkIncidentEvidence>, String> {
    let mut statement = connection
        .prepare(
            "SELECT source, status, checked_at, duration_ms, detail, observed_at
             FROM network_incident_evidence
             WHERE incident_id = ?1
             ORDER BY observed_at DESC, id DESC
             LIMIT ?2",
        )
        .map_err(|error| error.to_string())?;
    let rows = statement
        .query_map(
            params![incident_id, REPRESENTATIVE_EVIDENCE_LIMIT_SQL],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<i64>>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, String>(5)?,
                ))
            },
        )
        .map_err(|error| error.to_string())?;

    let mut evidence = Vec::new();
    for row in rows {
        let (source, status, checked_at, duration_ms, detail, observed_at) =
            row.map_err(|error| error.to_string())?;
        evidence.push(NetworkIncidentEvidence {
            source: EvidenceSource::from_str(&source)?,
            status: EvidenceStatus::from_str(&status)?,
            checked_at,
            duration_ms: duration_ms
                .map(u64::try_from)
                .transpose()
                .map_err(|_| "stored evidence duration is negative".to_string())?,
            detail,
            observed_at,
        });
    }
    Ok(evidence)
}

#[cfg_attr(not(test), allow(dead_code))]
fn prune_probe_observations(connection: &Connection, now: DateTime<Utc>) -> Result<(), String> {
    let cutoff = (now - Duration::hours(24)).to_rfc3339();
    connection
        .execute(
            "DELETE FROM network_probe_observations WHERE observed_at < ?1",
            params![cutoff],
        )
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Duration, Utc};

    fn evidence(source: EvidenceSource, status: EvidenceStatus) -> DiagnosticEvidence {
        DiagnosticEvidence {
            source,
            status,
            checked_at: Some("2026-06-30T00:00:00Z".into()),
            duration_ms: Some(10),
            detail: Some("test evidence".into()),
        }
    }

    #[test]
    fn creates_and_reads_latest_three_incidents() {
        let store = NetworkIncidentStore::open_in_memory().unwrap();
        let started = "2026-06-30T00:00:00Z";

        for index in 0..4 {
            store
                .create_incident(
                    DiagnosticArea::GatewayOrLocal,
                    &format!("2026-06-30T00:00:0{index}Z"),
                    "공유기 또는 로컬 연결 구간에서 시간 초과 근거가 반복 확인되었습니다.",
                    &[evidence(EvidenceSource::Gateway, EvidenceStatus::Timeout)],
                )
                .unwrap();
        }

        let incidents = store.recent_incidents(3).unwrap();
        assert_eq!(incidents.len(), 3);
        assert_eq!(incidents[0].started_at, "2026-06-30T00:00:03Z");
        assert_eq!(incidents[2].started_at, "2026-06-30T00:00:01Z");
        assert_eq!(incidents[0].status, NetworkIncidentStatus::Ongoing);
        assert_eq!(incidents[0].area, DiagnosticArea::GatewayOrLocal);
        assert_eq!(
            incidents[0].representative_evidence[0].source,
            EvidenceSource::Gateway
        );
        assert_ne!(incidents[0].started_at, started);
    }

    #[test]
    fn updates_existing_incident_to_resolved() {
        let store = NetworkIncidentStore::open_in_memory().unwrap();
        let id = store
            .create_incident(
                DiagnosticArea::Dns,
                "2026-06-30T00:00:00Z",
                "DNS에서 시간 초과 근거가 반복 확인되었습니다.",
                &[evidence(
                    EvidenceSource::DnsMicrosoft,
                    EvidenceStatus::Timeout,
                )],
            )
            .unwrap();

        store
            .update_incident(
                id,
                NetworkIncidentStatus::Recovering,
                DiagnosticArea::Dns,
                "2026-06-30T00:01:00Z",
                None,
                "DNS의 정상 근거를 추가 확인하고 있습니다.",
                &[evidence(
                    EvidenceSource::DnsMicrosoft,
                    EvidenceStatus::Success,
                )],
            )
            .unwrap();
        store
            .update_incident(
                id,
                NetworkIncidentStatus::Resolved,
                DiagnosticArea::Dns,
                "2026-06-30T00:02:00Z",
                Some("2026-06-30T00:02:00Z"),
                "DNS 장애가 복구되었습니다.",
                &[evidence(
                    EvidenceSource::DnsMicrosoft,
                    EvidenceStatus::Success,
                )],
            )
            .unwrap();

        let incident = store.recent_incidents(3).unwrap().remove(0);
        assert_eq!(incident.id, id);
        assert_eq!(incident.status, NetworkIncidentStatus::Resolved);
        assert_eq!(
            incident.resolved_at.as_deref(),
            Some("2026-06-30T00:02:00Z")
        );
        assert_eq!(incident.last_observed_at, "2026-06-30T00:02:00Z");
    }

    #[test]
    fn prunes_only_old_probe_observations() {
        let store = NetworkIncidentStore::open_in_memory().unwrap();
        let now: DateTime<Utc> = "2026-06-30T12:00:00Z".parse().unwrap();
        let old = (now - Duration::hours(25)).to_rfc3339();
        let fresh = (now - Duration::hours(1)).to_rfc3339();
        let incident_id = store
            .create_incident(
                DiagnosticArea::External,
                &old,
                "외부 연결 구간에서 실패 근거가 반복 확인되었습니다.",
                &[evidence(
                    EvidenceSource::HttpGoogle,
                    EvidenceStatus::Failure,
                )],
            )
            .unwrap();

        store
            .record_probe_observation(
                &old,
                Some("incident"),
                Some("external"),
                &[evidence(
                    EvidenceSource::HttpGoogle,
                    EvidenceStatus::Failure,
                )],
            )
            .unwrap();
        store
            .record_probe_observation(
                &fresh,
                Some("resolved"),
                Some("external"),
                &[evidence(
                    EvidenceSource::HttpGoogle,
                    EvidenceStatus::Success,
                )],
            )
            .unwrap();

        store.prune_probe_observations(now).unwrap();

        assert_eq!(store.probe_observation_count().unwrap(), 1);
        assert_eq!(store.recent_incidents(3).unwrap()[0].id, incident_id);
    }

    #[test]
    fn open_prunes_old_probe_observations_from_existing_db() {
        let path = std::env::temp_dir().join(format!(
            "pc-health-incident-store-{}-{}-startup.sqlite",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_file(&path);
        let now = Utc::now();
        let old = (now - Duration::hours(25)).to_rfc3339();
        let fresh = (now - Duration::hours(1)).to_rfc3339();

        {
            let store = NetworkIncidentStore::open(&path).unwrap();
            store
                .record_probe_observation(
                    &old,
                    Some("incident"),
                    Some("external"),
                    &[evidence(
                        EvidenceSource::HttpGoogle,
                        EvidenceStatus::Failure,
                    )],
                )
                .unwrap();
            store
                .record_probe_observation(
                    &fresh,
                    Some("resolved"),
                    Some("external"),
                    &[evidence(
                        EvidenceSource::HttpGoogle,
                        EvidenceStatus::Success,
                    )],
                )
                .unwrap();
        }

        let reopened = NetworkIncidentStore::open(&path).unwrap();

        assert_eq!(reopened.probe_observation_count().unwrap(), 1);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn repeated_updates_keep_only_representative_evidence_bound() {
        let store = NetworkIncidentStore::open_in_memory().unwrap();
        let id = store
            .create_incident(
                DiagnosticArea::Dns,
                "2026-06-30T00:00:00Z",
                "DNS에서 시간 초과 근거가 반복 확인되었습니다.",
                &[evidence(
                    EvidenceSource::DnsMicrosoft,
                    EvidenceStatus::Timeout,
                )],
            )
            .unwrap();

        for minute in 1..=6 {
            store
                .update_incident(
                    id,
                    NetworkIncidentStatus::Recovering,
                    DiagnosticArea::Dns,
                    &format!("2026-06-30T00:0{minute}:00Z"),
                    None,
                    "DNS의 정상 근거를 추가 확인하고 있습니다.",
                    &[evidence(
                        EvidenceSource::DnsMicrosoft,
                        EvidenceStatus::Success,
                    )],
                )
                .unwrap();
        }

        assert_eq!(store.incident_evidence_count(id).unwrap(), 4);
        assert_eq!(
            store.recent_incidents(3).unwrap()[0]
                .representative_evidence
                .len(),
            4
        );
    }
}
