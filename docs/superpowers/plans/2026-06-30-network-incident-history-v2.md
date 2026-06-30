# Network Incident History v2 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Persist confirmed network incidents in local SQLite and show the latest 3 incidents on the main PC Health screen.

**Architecture:** Keep lifecycle classification in the existing state machine and add a small recorder after each runtime status update. Store confirmed incidents indefinitely, store bounded probe observations for 24 hours, expose one fixed recent-incidents Tauri command, and render one feature-local recent incidents panel under the current status panel. Do not add a full history page, filters, export, settings, packet capture, traffic collection, router actions, or automatic recovery.

**Tech Stack:** Rust 2021, Tauri 2 managed state and commands, `rusqlite` with bundled SQLite, `chrono`, `serde_json`, React 19, TypeScript, Vitest, Testing Library, CSS Modules.

---

## File Structure

- Modify: `src-tauri/Cargo.toml`
  - Add `rusqlite = { version = "0.32", features = ["bundled"] }` for local SQLite persistence.
- Modify: `src-tauri/src/network/domain.rs`
  - Add serializable incident history contract types shared by Rust commands and frontend.
- Create: `src-tauri/src/network/incident_store.rs`
  - Own SQLite connection setup, migrations, incident insert/update/query, representative evidence storage, probe observation retention.
- Create: `src-tauri/src/network/incident_recorder.rs`
  - Own transition rules from previous/current `NetworkDiagnosticStatus` into store writes.
- Modify: `src-tauri/src/network/runtime.rs`
  - Accept an optional recorder and call it after each successful status update without letting storage failures stop diagnostics.
- Modify: `src-tauri/src/network/mod.rs`
  - Export the new store and recorder modules.
- Modify: `src-tauri/src/commands.rs`
  - Add a pure helper and Tauri command for latest 3 incidents.
- Modify: `src-tauri/src/lib.rs`
  - Initialize incident store in the Tauri app data directory and manage it for commands/runtime.
- Create: `src/features/network-incidents/types.ts`
  - Define frontend incident and evidence contracts matching Rust serialization.
- Create: `src/features/network-incidents/fixture.ts`
  - Provide empty, ongoing, recovering, resolved, and unknown fixtures.
- Create: `src/features/network-incidents/api.ts`
  - Wrap `get_recent_network_incidents` and browser capability detection.
- Create: `src/features/network-incidents/api.test.ts`
  - Verify the command name and browser capability behavior.
- Create: `src/features/network-incidents/RecentIncidentsPanel.tsx`
  - Fetch recent incidents, show empty/unavailable/error states, render latest 3 rows.
- Create: `src/features/network-incidents/RecentIncidentsPanel.module.css`
  - Quiet desktop utility styling for compact repeated rows.
- Create: `src/features/network-incidents/RecentIncidentsPanel.test.tsx`
  - Verify empty, records, max 3, status labels, unknown area, and command failure behavior.
- Modify: `src/features/product-home/ProductHome.tsx`
  - Place `RecentIncidentsPanel` between `NetworkStatusPanel` and `NetworkProbePanel`.
- Modify: `src/features/product-home/ProductHome.test.tsx`
  - Mock the new panel and verify main page ordering/inclusion.

---

### Task 1: SQLite Store Contract And Persistence

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/network/domain.rs`
- Create: `src-tauri/src/network/incident_store.rs`
- Modify: `src-tauri/src/network/mod.rs`

- [ ] **Step 1: Add failing store tests**

Create `src-tauri/src/network/incident_store.rs` with only test scaffolding and the expected API usage:

```rust
use super::domain::{
    DiagnosticArea, DiagnosticEvidence, EvidenceSource, EvidenceStatus, NetworkIncidentStatus,
};

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
                &[evidence(EvidenceSource::DnsMicrosoft, EvidenceStatus::Timeout)],
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
                &[evidence(EvidenceSource::DnsMicrosoft, EvidenceStatus::Success)],
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
                &[evidence(EvidenceSource::DnsMicrosoft, EvidenceStatus::Success)],
            )
            .unwrap();

        let incident = store.recent_incidents(3).unwrap().remove(0);
        assert_eq!(incident.id, id);
        assert_eq!(incident.status, NetworkIncidentStatus::Resolved);
        assert_eq!(incident.resolved_at.as_deref(), Some("2026-06-30T00:02:00Z"));
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
                &[evidence(EvidenceSource::HttpGoogle, EvidenceStatus::Failure)],
            )
            .unwrap();

        store
            .record_probe_observation(
                &old,
                Some("incident"),
                Some("external"),
                &[evidence(EvidenceSource::HttpGoogle, EvidenceStatus::Failure)],
            )
            .unwrap();
        store
            .record_probe_observation(
                &fresh,
                Some("resolved"),
                Some("external"),
                &[evidence(EvidenceSource::HttpGoogle, EvidenceStatus::Success)],
            )
            .unwrap();

        store.prune_probe_observations(now).unwrap();

        assert_eq!(store.probe_observation_count().unwrap(), 1);
        assert_eq!(store.recent_incidents(3).unwrap()[0].id, incident_id);
    }
}
```

- [ ] **Step 2: Run the focused Rust test and verify it fails**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml network::incident_store
```

Expected: FAIL because `NetworkIncidentStore`, incident types, and module export do not exist.

- [ ] **Step 3: Add incident contract types**

In `src-tauri/src/network/domain.rs`, extend the serde import and add these types after `NetworkDiagnosticStatus`:

```rust
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkIncidentStatus {
    Ongoing,
    Recovering,
    Resolved,
}

impl NetworkIncidentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ongoing => "ongoing",
            Self::Recovering => "recovering",
            Self::Resolved => "resolved",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkIncidentEvidence {
    pub source: EvidenceSource,
    pub status: EvidenceStatus,
    pub checked_at: Option<String>,
    pub duration_ms: Option<u64>,
    pub detail: Option<String>,
    pub observed_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkIncident {
    pub id: i64,
    pub status: NetworkIncidentStatus,
    pub area: DiagnosticArea,
    pub started_at: String,
    pub last_observed_at: String,
    pub resolved_at: Option<String>,
    pub summary: String,
    pub representative_evidence: Vec<NetworkIncidentEvidence>,
}
```

Also add `Deserialize` to these existing derives because the store will deserialize JSON evidence in tests and reads:

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
```

Apply that derive shape to `DiagnosticLifecycle`, `DiagnosticArea`, `EvidenceSource`, `EvidenceStatus`, and `DiagnosticEvidence`.

- [ ] **Step 4: Add SQLite dependency and module export**

In `src-tauri/Cargo.toml`, add:

```toml
rusqlite = { version = "0.32", features = ["bundled"] }
```

In `src-tauri/src/network/mod.rs`, add:

```rust
pub mod incident_store;
```

- [ ] **Step 5: Implement the store**

Replace `src-tauri/src/network/incident_store.rs` with:

```rust
use super::domain::{
    DiagnosticArea, DiagnosticEvidence, EvidenceSource, EvidenceStatus, NetworkIncident,
    NetworkIncidentEvidence, NetworkIncidentStatus,
};
use chrono::{DateTime, Duration, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use std::{
    path::Path,
    sync::{Arc, Mutex},
};

#[derive(Clone)]
pub struct NetworkIncidentStore {
    connection: Arc<Mutex<Connection>>,
}

impl NetworkIncidentStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        let connection = Connection::open(path).map_err(|error| error.to_string())?;
        let store = Self {
            connection: Arc::new(Mutex::new(connection)),
        };
        store.migrate()?;
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
        let connection = self.connection.lock().map_err(|_| "incident store lock failed")?;
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
        let mut connection = self.connection.lock().map_err(|_| "incident store lock failed")?;
        let transaction = connection.transaction().map_err(|error| error.to_string())?;
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
        let mut connection = self.connection.lock().map_err(|_| "incident store lock failed")?;
        let transaction = connection.transaction().map_err(|error| error.to_string())?;
        transaction
            .execute(
                "UPDATE network_incidents
                 SET status = ?1, area = ?2, last_observed_at = ?3, resolved_at = ?4, summary = ?5
                 WHERE id = ?6",
                params![status.as_str(), area.as_str(), observed_at, resolved_at, summary, id],
            )
            .map_err(|error| error.to_string())?;
        insert_evidence(&transaction, id, observed_at, evidence)?;
        transaction.commit().map_err(|error| error.to_string())
    }

    pub fn recent_incidents(&self, limit: usize) -> Result<Vec<NetworkIncident>, String> {
        let connection = self.connection.lock().map_err(|_| "incident store lock failed")?;
        let mut statement = connection
            .prepare(
                "SELECT id, status, area, started_at, last_observed_at, resolved_at, summary
                 FROM network_incidents
                 ORDER BY started_at DESC, id DESC
                 LIMIT ?1",
            )
            .map_err(|error| error.to_string())?;
        let incident_rows = statement
            .query_map(params![limit as i64], |row| {
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

        let mut incidents = vec![];
        for row in incident_rows {
            let (id, status, area, started_at, last_observed_at, resolved_at, summary) =
                row.map_err(|error| error.to_string())?;
            incidents.push(NetworkIncident {
                id,
                status: parse_incident_status(&status)?,
                area: DiagnosticArea::from_str(&area)?,
                started_at,
                last_observed_at,
                resolved_at,
                summary,
                representative_evidence: self.incident_evidence_locked(&connection, id)?,
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
        let connection = self.connection.lock().map_err(|_| "incident store lock failed")?;
        connection
            .execute(
                "INSERT INTO network_probe_observations (observed_at, lifecycle, suspected_area, evidence_json)
                 VALUES (?1, ?2, ?3, ?4)",
                params![observed_at, lifecycle, suspected_area, evidence_json],
            )
            .map_err(|error| error.to_string())?;
        if let Ok(now) = DateTime::parse_from_rfc3339(observed_at) {
            let _ = self.prune_probe_observations_locked(&connection, now.with_timezone(&Utc));
        }
        Ok(())
    }

    pub fn prune_probe_observations(&self, now: DateTime<Utc>) -> Result<(), String> {
        let connection = self.connection.lock().map_err(|_| "incident store lock failed")?;
        self.prune_probe_observations_locked(&connection, now)
    }

    fn prune_probe_observations_locked(
        &self,
        connection: &Connection,
        now: DateTime<Utc>,
    ) -> Result<(), String> {
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
    pub fn probe_observation_count(&self) -> Result<i64, String> {
        let connection = self.connection.lock().map_err(|_| "incident store lock failed")?;
        connection
            .query_row("SELECT COUNT(*) FROM network_probe_observations", [], |row| row.get(0))
            .map_err(|error| error.to_string())
    }

    fn incident_evidence_locked(
        &self,
        connection: &Connection,
        incident_id: i64,
    ) -> Result<Vec<NetworkIncidentEvidence>, String> {
        let mut statement = connection
            .prepare(
                "SELECT source, status, checked_at, duration_ms, detail, observed_at
                 FROM network_incident_evidence
                 WHERE incident_id = ?1
                 ORDER BY observed_at DESC, id DESC
                 LIMIT 4",
            )
            .map_err(|error| error.to_string())?;
        let rows = statement
            .query_map(params![incident_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<u64>>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, String>(5)?,
                ))
            })
            .map_err(|error| error.to_string())?;

        let mut evidence = vec![];
        for row in rows {
            let (source, status, checked_at, duration_ms, detail, observed_at) =
                row.map_err(|error| error.to_string())?;
            evidence.push(NetworkIncidentEvidence {
                source: EvidenceSource::from_str(&source)?,
                status: EvidenceStatus::from_str(&status)?,
                checked_at,
                duration_ms,
                detail,
                observed_at,
            });
        }
        Ok(evidence)
    }
}

fn insert_evidence(
    transaction: &rusqlite::Transaction<'_>,
    incident_id: i64,
    observed_at: &str,
    evidence: &[DiagnosticEvidence],
) -> Result<(), String> {
    for item in evidence.iter().take(4) {
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
                    item.duration_ms,
                    item.detail,
                    observed_at,
                ],
            )
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn parse_incident_status(value: &str) -> Result<NetworkIncidentStatus, String> {
    match value {
        "ongoing" => Ok(NetworkIncidentStatus::Ongoing),
        "recovering" => Ok(NetworkIncidentStatus::Recovering),
        "resolved" => Ok(NetworkIncidentStatus::Resolved),
        other => Err(format!("unknown incident status: {other}")),
    }
}
```

In `domain.rs`, add `as_str` and `from_str` methods for `DiagnosticArea`, `EvidenceSource`, and `EvidenceStatus` using the same snake_case strings already serialized to JSON.

- [ ] **Step 6: Verify store tests pass**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml network::incident_store
```

Expected: PASS.

- [ ] **Step 7: Commit Task 1**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/network/domain.rs src-tauri/src/network/incident_store.rs src-tauri/src/network/mod.rs
git commit -m "feat: add network incident store"
```

---

### Task 2: Incident Recorder And Runtime Hook

**Files:**
- Create: `src-tauri/src/network/incident_recorder.rs`
- Modify: `src-tauri/src/network/runtime.rs`
- Modify: `src-tauri/src/network/mod.rs`

- [ ] **Step 1: Add failing recorder tests**

Create `src-tauri/src/network/incident_recorder.rs` with tests that exercise transitions:

```rust
use super::{
    domain::{
        DiagnosticArea, DiagnosticEvidence, DiagnosticLifecycle, EvidenceSource, EvidenceStatus,
        NetworkDiagnosticStatus, NetworkIncidentStatus, RuntimeAvailability,
    },
    incident_store::NetworkIncidentStore,
};

#[derive(Clone)]
pub struct NetworkIncidentRecorder {
    store: NetworkIncidentStore,
    open_incident_id: Option<i64>,
    previous: Option<NetworkDiagnosticStatus>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn status(
        lifecycle: DiagnosticLifecycle,
        area: Option<DiagnosticArea>,
        observed_at: &str,
        evidence_status: EvidenceStatus,
    ) -> NetworkDiagnosticStatus {
        NetworkDiagnosticStatus {
            availability: RuntimeAvailability::Running,
            lifecycle: Some(lifecycle),
            suspected_area: area,
            observed_at: Some(observed_at.into()),
            last_full_probe_at: Some(observed_at.into()),
            evidence: vec![DiagnosticEvidence {
                source: EvidenceSource::Gateway,
                status: evidence_status,
                checked_at: Some(observed_at.into()),
                duration_ms: Some(10),
                detail: Some("gateway check".into()),
            }],
            error: None,
        }
    }

    #[test]
    fn suspected_does_not_create_incident_until_confirmed() {
        let store = NetworkIncidentStore::open_in_memory().unwrap();
        let mut recorder = NetworkIncidentRecorder::new(store.clone());

        recorder
            .record(&status(
                DiagnosticLifecycle::Suspected,
                Some(DiagnosticArea::GatewayOrLocal),
                "2026-06-30T00:00:00Z",
                EvidenceStatus::Timeout,
            ))
            .unwrap();

        assert!(store.recent_incidents(3).unwrap().is_empty());
    }

    #[test]
    fn incident_recovery_and_resolved_update_same_record() {
        let store = NetworkIncidentStore::open_in_memory().unwrap();
        let mut recorder = NetworkIncidentRecorder::new(store.clone());

        recorder
            .record(&status(
                DiagnosticLifecycle::Suspected,
                Some(DiagnosticArea::GatewayOrLocal),
                "2026-06-30T00:00:00Z",
                EvidenceStatus::Timeout,
            ))
            .unwrap();
        recorder
            .record(&status(
                DiagnosticLifecycle::Incident,
                Some(DiagnosticArea::GatewayOrLocal),
                "2026-06-30T00:00:20Z",
                EvidenceStatus::Timeout,
            ))
            .unwrap();
        recorder
            .record(&status(
                DiagnosticLifecycle::Recovering,
                Some(DiagnosticArea::GatewayOrLocal),
                "2026-06-30T00:00:40Z",
                EvidenceStatus::Success,
            ))
            .unwrap();
        recorder
            .record(&status(
                DiagnosticLifecycle::Resolved,
                None,
                "2026-06-30T00:01:00Z",
                EvidenceStatus::Success,
            ))
            .unwrap();

        let incidents = store.recent_incidents(3).unwrap();
        assert_eq!(incidents.len(), 1);
        assert_eq!(incidents[0].status, NetworkIncidentStatus::Resolved);
        assert_eq!(incidents[0].resolved_at.as_deref(), Some("2026-06-30T00:01:00Z"));
    }

    #[test]
    fn resolved_then_new_incident_creates_new_record() {
        let store = NetworkIncidentStore::open_in_memory().unwrap();
        let mut recorder = NetworkIncidentRecorder::new(store.clone());

        for (lifecycle, at) in [
            (DiagnosticLifecycle::Incident, "2026-06-30T00:00:00Z"),
            (DiagnosticLifecycle::Resolved, "2026-06-30T00:00:20Z"),
            (DiagnosticLifecycle::Suspected, "2026-06-30T00:00:40Z"),
            (DiagnosticLifecycle::Incident, "2026-06-30T00:01:00Z"),
        ] {
            recorder
                .record(&status(
                    lifecycle,
                    Some(DiagnosticArea::GatewayOrLocal),
                    at,
                    EvidenceStatus::Timeout,
                ))
                .unwrap();
        }

        assert_eq!(store.recent_incidents(3).unwrap().len(), 2);
    }
}
```

- [ ] **Step 2: Run recorder tests and verify they fail**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml network::incident_recorder
```

Expected: FAIL because the recorder implementation is missing.

- [ ] **Step 3: Implement recorder transition logic**

In `src-tauri/src/network/incident_recorder.rs`, keep the test helper imports and implement:

```rust
impl NetworkIncidentRecorder {
    pub fn new(store: NetworkIncidentStore) -> Self {
        Self {
            store,
            open_incident_id: None,
            previous: None,
        }
    }

    pub fn record(&mut self, status: &NetworkDiagnosticStatus) -> Result<(), String> {
        let Some(observed_at) = status.observed_at.as_deref() else {
            self.previous = Some(status.clone());
            return Ok(());
        };

        self.store.record_probe_observation(
            observed_at,
            status.lifecycle.as_ref().map(DiagnosticLifecycle::as_str),
            status.suspected_area.as_ref().map(DiagnosticArea::as_str),
            &status.evidence,
        )?;

        match status.lifecycle {
            Some(DiagnosticLifecycle::Incident) => self.record_incident(status, observed_at)?,
            Some(DiagnosticLifecycle::Recovering) => {
                self.update_open(status, NetworkIncidentStatus::Recovering, observed_at, None)?
            }
            Some(DiagnosticLifecycle::Resolved) => self.update_open(
                status,
                NetworkIncidentStatus::Resolved,
                observed_at,
                Some(observed_at),
            )?,
            Some(DiagnosticLifecycle::Normal | DiagnosticLifecycle::Suspected) | None => {}
        }

        if matches!(status.lifecycle, Some(DiagnosticLifecycle::Resolved)) {
            self.open_incident_id = None;
        }
        self.previous = Some(status.clone());
        Ok(())
    }

    fn record_incident(
        &mut self,
        status: &NetworkDiagnosticStatus,
        observed_at: &str,
    ) -> Result<(), String> {
        let area = status.suspected_area.clone().unwrap_or(DiagnosticArea::Unknown);
        let summary = summary_for(status, NetworkIncidentStatus::Ongoing);
        if let Some(id) = self.open_incident_id {
            return self.store.update_incident(
                id,
                NetworkIncidentStatus::Ongoing,
                area,
                observed_at,
                None,
                &summary,
                &status.evidence,
            );
        }
        let id = self
            .store
            .create_incident(area, observed_at, &summary, &status.evidence)?;
        self.open_incident_id = Some(id);
        Ok(())
    }

    fn update_open(
        &mut self,
        status: &NetworkDiagnosticStatus,
        incident_status: NetworkIncidentStatus,
        observed_at: &str,
        resolved_at: Option<&str>,
    ) -> Result<(), String> {
        let Some(id) = self.open_incident_id else {
            self.previous = Some(status.clone());
            return Ok(());
        };
        let area = status
            .suspected_area
            .clone()
            .or_else(|| self.previous.as_ref().and_then(|previous| previous.suspected_area.clone()))
            .unwrap_or(DiagnosticArea::Unknown);
        let summary = summary_for(status, incident_status.clone());
        self.store.update_incident(
            id,
            incident_status,
            area,
            observed_at,
            resolved_at,
            &summary,
            &status.evidence,
        )
    }
}
```

Add `summary_for` with the exact Korean strings:

```rust
fn summary_for(status: &NetworkDiagnosticStatus, incident_status: NetworkIncidentStatus) -> String {
    let area = status.suspected_area.clone().unwrap_or(DiagnosticArea::Unknown);
    let area_label = match area {
        DiagnosticArea::LocalConnection => "로컬 연결 구간",
        DiagnosticArea::GatewayOrLocal => "공유기 또는 로컬 연결 구간",
        DiagnosticArea::Dns => "DNS",
        DiagnosticArea::External => "외부 연결 구간",
        DiagnosticArea::Unknown => "확인 불가",
    };
    match incident_status {
        NetworkIncidentStatus::Ongoing => {
            format!("{area_label}에서 이상 근거가 반복 확인되었습니다.")
        }
        NetworkIncidentStatus::Recovering => {
            format!("{area_label}의 정상 근거를 추가 확인하고 있습니다.")
        }
        NetworkIncidentStatus::Resolved => "네트워크 장애가 복구되었습니다.".into(),
    }
}
```

In `domain.rs`, add `as_str` for `DiagnosticLifecycle`.

In `src-tauri/src/network/mod.rs`, add:

```rust
pub mod incident_recorder;
```

- [ ] **Step 4: Hook recorder into runtime**

In `runtime.rs`, add `incident_recorder::NetworkIncidentRecorder` behind the same Windows/test cfg used by worker code. Change `NetworkDiagnosticsRuntime::platform` on Windows to accept a recorder:

```rust
pub fn platform(
    coordinator: NetworkProbeCoordinator,
    recorder: NetworkIncidentRecorder,
) -> Self {
    let (wait, stop) = SystemWait::new();
    Self::start(coordinator, wait, stop, Some(recorder))
}
```

Keep the non-Windows platform signature accepting `_recorder: NetworkIncidentRecorder` and returning unavailable.

Thread `Option<NetworkIncidentRecorder>` through `start`, `run_worker`, `run_worker_observed`, and `run_probe`. After `let status = machine.apply(...)`, call recorder before writing latest:

```rust
if let Some(recorder) = recorder.as_mut() {
    let _ = recorder.record(&status);
}
```

Do not convert recorder errors into `RuntimeAvailability::Error`; storage failure must not stop current diagnostics.

- [ ] **Step 5: Verify runtime and recorder tests pass**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml network::incident_recorder network::runtime
```

Expected: PASS.

- [ ] **Step 6: Commit Task 2**

```bash
git add src-tauri/src/network/incident_recorder.rs src-tauri/src/network/runtime.rs src-tauri/src/network/mod.rs src-tauri/src/network/domain.rs
git commit -m "feat: record network diagnostic incidents"
```

---

### Task 3: Tauri Command And App Initialization

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add failing command helper test**

In `src-tauri/src/commands.rs`, import `NetworkIncident` and `NetworkIncidentStore`. Add a pure helper:

```rust
fn latest_network_incidents(
    store: &NetworkIncidentStore,
) -> Result<Vec<NetworkIncident>, String> {
    store.recent_incidents(3)
}
```

Add this test:

```rust
#[test]
fn recent_incidents_command_limits_to_three() {
    use crate::network::{
        domain::{DiagnosticArea, DiagnosticEvidence, EvidenceSource, EvidenceStatus},
        incident_store::NetworkIncidentStore,
    };

    let store = NetworkIncidentStore::open_in_memory().unwrap();
    for index in 0..4 {
        store
            .create_incident(
                DiagnosticArea::External,
                &format!("2026-06-30T00:00:0{index}Z"),
                "외부 연결 구간에서 이상 근거가 반복 확인되었습니다.",
                &[DiagnosticEvidence {
                    source: EvidenceSource::HttpGoogle,
                    status: EvidenceStatus::Timeout,
                    checked_at: Some("2026-06-30T00:00:00Z".into()),
                    duration_ms: Some(10),
                    detail: Some("test".into()),
                }],
            )
            .unwrap();
    }

    assert_eq!(latest_network_incidents(&store).unwrap().len(), 3);
}
```

- [ ] **Step 2: Run command tests and verify failure**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml commands
```

Expected: FAIL until imports and command registration compile.

- [ ] **Step 3: Add Tauri command**

In `commands.rs`, add:

```rust
#[tauri::command]
pub fn get_recent_network_incidents(
    store: tauri::State<'_, NetworkIncidentStore>,
) -> Result<Vec<NetworkIncident>, String> {
    latest_network_incidents(store.inner())
}
```

- [ ] **Step 4: Initialize store in app setup**

In `src-tauri/src/lib.rs`, replace the current file with setup-managed initialization:

```rust
mod commands;
mod network;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let coordinator = network::service::NetworkProbeCoordinator::platform();
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|error| format!("failed to resolve app data dir: {error}"))?;
            std::fs::create_dir_all(&app_data_dir)
                .map_err(|error| format!("failed to create app data dir: {error}"))?;
            let store = network::incident_store::NetworkIncidentStore::open(
                app_data_dir.join("pc-health.db"),
            )?;
            let recorder =
                network::incident_recorder::NetworkIncidentRecorder::new(store.clone());
            let runtime =
                network::runtime::NetworkDiagnosticsRuntime::platform(coordinator.clone(), recorder);
            app.manage(coordinator);
            app.manage(store);
            app.manage(runtime);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_network_probe_snapshot,
            commands::get_network_diagnostic_status,
            commands::get_recent_network_incidents,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

This keeps the DB under the OS-managed app data directory rather than the repository or current working directory.

- [ ] **Step 5: Verify command tests and cargo check**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml commands
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: PASS.

- [ ] **Step 6: Commit Task 3**

```bash
git add src-tauri/src/commands.rs src-tauri/src/lib.rs
git commit -m "feat: expose recent network incidents command"
```

---

### Task 4: Frontend Incident API And Panel

**Files:**
- Create: `src/features/network-incidents/types.ts`
- Create: `src/features/network-incidents/fixture.ts`
- Create: `src/features/network-incidents/api.ts`
- Create: `src/features/network-incidents/api.test.ts`
- Create: `src/features/network-incidents/RecentIncidentsPanel.tsx`
- Create: `src/features/network-incidents/RecentIncidentsPanel.module.css`
- Create: `src/features/network-incidents/RecentIncidentsPanel.test.tsx`

- [ ] **Step 1: Add frontend types and API tests**

Create `src/features/network-incidents/types.ts`:

```ts
import type {
  DiagnosticArea,
  DiagnosticEvidence,
} from "@/features/network-diagnostics/types";

export type NetworkIncidentStatus = "ongoing" | "recovering" | "resolved";

export interface NetworkIncidentEvidence extends DiagnosticEvidence {
  observedAt: string;
}

export interface NetworkIncident {
  id: number;
  status: NetworkIncidentStatus;
  area: DiagnosticArea;
  startedAt: string;
  lastObservedAt: string;
  resolvedAt: string | null;
  summary: string;
  representativeEvidence: NetworkIncidentEvidence[];
}
```

Create `src/features/network-incidents/api.test.ts`:

```ts
import { beforeEach, describe, expect, it, vi } from "vitest";
import { resolvedIncidentFixture } from "./fixture";

const { invokeMock, isTauriMock } = vi.hoisted(() => ({
  invokeMock: vi.fn(),
  isTauriMock: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
  isTauri: isTauriMock,
}));

import { canUseNetworkIncidents, getRecentNetworkIncidents } from "./api";

describe("network incidents API", () => {
  beforeEach(() => {
    invokeMock.mockReset();
    isTauriMock.mockReset();
  });

  it("reads recent incidents from Tauri", async () => {
    invokeMock.mockResolvedValue([resolvedIncidentFixture]);

    await expect(getRecentNetworkIncidents()).resolves.toEqual([
      resolvedIncidentFixture,
    ]);
    expect(invokeMock).toHaveBeenCalledWith("get_recent_network_incidents");
  });

  it("reports browser mode without invoking Tauri", () => {
    isTauriMock.mockReturnValue(false);

    expect(canUseNetworkIncidents()).toBe(false);
    expect(invokeMock).not.toHaveBeenCalled();
  });
});
```

- [ ] **Step 2: Add fixtures and API implementation**

Create `src/features/network-incidents/fixture.ts`:

```ts
import type { NetworkIncident } from "./types";

export const ongoingIncidentFixture: NetworkIncident = {
  id: 3,
  status: "ongoing",
  area: "gateway_or_local",
  startedAt: "2026-06-30T00:02:00Z",
  lastObservedAt: "2026-06-30T00:02:20Z",
  resolvedAt: null,
  summary: "공유기 또는 로컬 연결 구간에서 이상 근거가 반복 확인되었습니다.",
  representativeEvidence: [
    {
      source: "gateway",
      status: "timeout",
      checkedAt: "2026-06-30T00:02:20Z",
      durationMs: 1000,
      detail: "gateway timeout",
      observedAt: "2026-06-30T00:02:20Z",
    },
  ],
};

export const recoveringIncidentFixture: NetworkIncident = {
  ...ongoingIncidentFixture,
  id: 2,
  status: "recovering",
  lastObservedAt: "2026-06-30T00:01:20Z",
  summary: "공유기 또는 로컬 연결 구간의 정상 근거를 추가 확인하고 있습니다.",
};

export const resolvedIncidentFixture: NetworkIncident = {
  ...ongoingIncidentFixture,
  id: 1,
  status: "resolved",
  startedAt: "2026-06-30T00:00:00Z",
  lastObservedAt: "2026-06-30T00:00:40Z",
  resolvedAt: "2026-06-30T00:00:40Z",
  summary: "네트워크 장애가 복구되었습니다.",
};

export const unknownIncidentFixture: NetworkIncident = {
  ...ongoingIncidentFixture,
  id: 4,
  area: "unknown",
  summary: "확인 불가 구간에서 이상 근거가 반복 확인되었습니다.",
};
```

Create `src/features/network-incidents/api.ts`:

```ts
import { invoke, isTauri } from "@tauri-apps/api/core";
import type { NetworkIncident } from "./types";

export const canUseNetworkIncidents = () => isTauri();
export const getRecentNetworkIncidents = () =>
  invoke<NetworkIncident[]>("get_recent_network_incidents");
```

- [ ] **Step 3: Add failing panel tests**

Create `src/features/network-incidents/RecentIncidentsPanel.test.tsx` with:

```tsx
import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  ongoingIncidentFixture,
  recoveringIncidentFixture,
  resolvedIncidentFixture,
  unknownIncidentFixture,
} from "./fixture";
import RecentIncidentsPanel from "./RecentIncidentsPanel";

const { canUseMock, getRecentMock } = vi.hoisted(() => ({
  canUseMock: vi.fn(),
  getRecentMock: vi.fn(),
}));

vi.mock("./api", () => ({
  canUseNetworkIncidents: canUseMock,
  getRecentNetworkIncidents: getRecentMock,
}));

describe("RecentIncidentsPanel", () => {
  beforeEach(() => {
    canUseMock.mockReturnValue(true);
    getRecentMock.mockReset();
  });

  it("shows an empty state when there are no saved incidents", async () => {
    getRecentMock.mockResolvedValue([]);

    render(<RecentIncidentsPanel />);

    expect(
      await screen.findByText("저장된 장애 기록이 없습니다."),
    ).toBeInTheDocument();
  });

  it("renders recent incidents with stable status and area labels", async () => {
    getRecentMock.mockResolvedValue([
      ongoingIncidentFixture,
      recoveringIncidentFixture,
      resolvedIncidentFixture,
      unknownIncidentFixture,
    ]);

    render(<RecentIncidentsPanel />);

    expect(await screen.findByText("진행 중")).toBeInTheDocument();
    expect(screen.getByText("복구 확인 중")).toBeInTheDocument();
    expect(screen.getByText("복구됨")).toBeInTheDocument();
    expect(screen.getAllByText("공유기 또는 로컬 연결 구간").length).toBeGreaterThan(0);
    expect(screen.queryByText("확인 불가 구간에서 이상 근거가 반복 확인되었습니다.")).not.toBeInTheDocument();
  });

  it("limits the main panel to three rows", async () => {
    getRecentMock.mockResolvedValue([
      ongoingIncidentFixture,
      recoveringIncidentFixture,
      resolvedIncidentFixture,
      unknownIncidentFixture,
    ]);

    render(<RecentIncidentsPanel />);

    expect(await screen.findAllByRole("listitem")).toHaveLength(3);
  });

  it("shows unavailable state in browser mode", async () => {
    canUseMock.mockReturnValue(false);

    render(<RecentIncidentsPanel />);

    expect(
      await screen.findByText("장애 기록을 불러올 수 없습니다."),
    ).toBeInTheDocument();
    expect(getRecentMock).not.toHaveBeenCalled();
  });

  it("shows command failure without claiming a current outage", async () => {
    getRecentMock.mockRejectedValue(new Error("db locked"));

    render(<RecentIncidentsPanel />);

    expect(
      await screen.findByText("장애 기록을 불러올 수 없습니다."),
    ).toBeInTheDocument();
    expect(screen.queryByText("장애 확인")).not.toBeInTheDocument();
  });
});
```

- [ ] **Step 4: Implement panel and CSS**

Create `RecentIncidentsPanel.tsx`:

```tsx
import { useEffect, useState } from "react";
import { canUseNetworkIncidents, getRecentNetworkIncidents } from "./api";
import type { NetworkIncident, NetworkIncidentStatus } from "./types";
import styles from "./RecentIncidentsPanel.module.css";

const statusLabels: Record<NetworkIncidentStatus, string> = {
  ongoing: "진행 중",
  recovering: "복구 확인 중",
  resolved: "복구됨",
};

const areaLabels = {
  local_connection: "로컬 연결 구간",
  gateway_or_local: "공유기 또는 로컬 연결 구간",
  dns: "DNS",
  external: "외부 연결 구간",
  unknown: "확인 불가",
};

function formatTime(value: string) {
  return new Intl.DateTimeFormat("ko-KR", {
    dateStyle: "short",
    timeStyle: "medium",
  }).format(new Date(value));
}

export default function RecentIncidentsPanel() {
  const [available] = useState(canUseNetworkIncidents);
  const [incidents, setIncidents] = useState<NetworkIncident[]>([]);
  const [failed, setFailed] = useState(!available);
  const [loading, setLoading] = useState(available);

  useEffect(() => {
    if (!available) return;
    let cancelled = false;
    getRecentNetworkIncidents()
      .then((items) => {
        if (!cancelled) setIncidents(items.slice(0, 3));
      })
      .catch(() => {
        if (!cancelled) setFailed(true);
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, [available]);

  return (
    <section className={styles["panel"]} aria-labelledby="recent-incidents-title">
      <header className={styles["header"]}>
        <p>RECENT INCIDENTS</p>
        <h2 id="recent-incidents-title">최근 장애</h2>
      </header>
      {loading ? <p className={styles["muted"]}>장애 기록 확인 중</p> : null}
      {!loading && failed ? (
        <p className={styles["muted"]}>장애 기록을 불러올 수 없습니다.</p>
      ) : null}
      {!loading && !failed && incidents.length === 0 ? (
        <p className={styles["muted"]}>저장된 장애 기록이 없습니다.</p>
      ) : null}
      {!loading && !failed && incidents.length > 0 ? (
        <ul className={styles["list"]}>
          {incidents.map((incident) => (
            <li className={styles["row"]} key={incident.id}>
              <div>
                <strong>{statusLabels[incident.status]}</strong>
                <span>{areaLabels[incident.area]}</span>
              </div>
              <p>{incident.summary}</p>
              <dl>
                <div>
                  <dt>시작</dt>
                  <dd><time dateTime={incident.startedAt}>{formatTime(incident.startedAt)}</time></dd>
                </div>
                <div>
                  <dt>{incident.resolvedAt ? "복구" : "마지막 확인"}</dt>
                  <dd>
                    <time dateTime={incident.resolvedAt ?? incident.lastObservedAt}>
                      {formatTime(incident.resolvedAt ?? incident.lastObservedAt)}
                    </time>
                  </dd>
                </div>
              </dl>
            </li>
          ))}
        </ul>
      ) : null}
    </section>
  );
}
```

Create compact CSS matching the existing panel tone:

```css
.panel {
  border: 1px solid rgba(148, 163, 184, 0.2);
  border-radius: 8px;
  padding: 22px;
  background: rgba(15, 23, 42, 0.62);
}

.header {
  display: grid;
  gap: 6px;
}

.header p {
  margin: 0;
  color: #8ea4bd;
  font-size: 0.72rem;
  font-weight: 700;
  letter-spacing: 0;
}

.header h2 {
  margin: 0;
  color: #f8fafc;
  font-size: 1.1rem;
}

.muted {
  margin: 16px 0 0;
  color: #94a3b8;
}

.list {
  display: grid;
  gap: 10px;
  margin: 18px 0 0;
  padding: 0;
  list-style: none;
}

.row {
  display: grid;
  gap: 10px;
  border: 1px solid rgba(148, 163, 184, 0.16);
  border-radius: 8px;
  padding: 14px;
  background: rgba(2, 6, 23, 0.26);
}

.row > div:first-child {
  display: flex;
  flex-wrap: wrap;
  gap: 8px 12px;
  align-items: center;
}

.row strong {
  color: #f8fafc;
}

.row span,
.row p,
.row dt {
  color: #94a3b8;
}

.row p {
  margin: 0;
}

.row dl {
  display: flex;
  flex-wrap: wrap;
  gap: 10px 18px;
  margin: 0;
}

.row dl div {
  display: grid;
  gap: 2px;
}

.row dt,
.row dd {
  margin: 0;
  font-size: 0.82rem;
}

.row dd {
  color: #dbeafe;
}
```

- [ ] **Step 5: Verify frontend panel tests**

Run:

```bash
npm test -- src/features/network-incidents
```

Expected: PASS.

- [ ] **Step 6: Commit Task 4**

```bash
git add src/features/network-incidents
git commit -m "feat: add recent network incidents panel"
```

---

### Task 5: Main Screen Integration And Full Verification

**Files:**
- Modify: `src/features/product-home/ProductHome.tsx`
- Modify: `src/features/product-home/ProductHome.test.tsx`

- [ ] **Step 1: Add failing ProductHome test expectation**

In `ProductHome.test.tsx`, add a mock:

```tsx
vi.mock("@/features/network-incidents/RecentIncidentsPanel", () => ({
  default: () => <section>최근 장애</section>,
}));
```

Add assertion:

```tsx
expect(screen.getByText("최근 장애")).toBeInTheDocument();
```

- [ ] **Step 2: Run focused ProductHome test and verify failure**

Run:

```bash
npm test -- src/features/product-home/ProductHome.test.tsx
```

Expected: FAIL because `ProductHome` does not render `RecentIncidentsPanel`.

- [ ] **Step 3: Render recent incidents between status and raw probe**

In `ProductHome.tsx`, import:

```tsx
import RecentIncidentsPanel from "@/features/network-incidents/RecentIncidentsPanel";
```

Render:

```tsx
<NetworkStatusPanel />
<RecentIncidentsPanel />
<NetworkProbePanel />
```

- [ ] **Step 4: Verify focused frontend integration**

Run:

```bash
npm test -- src/features/product-home/ProductHome.test.tsx
```

Expected: PASS.

- [ ] **Step 5: Run full verification**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
npm test
npm run build
```

Expected: all commands PASS.

- [ ] **Step 6: Commit Task 5**

```bash
git add src/features/product-home/ProductHome.tsx src/features/product-home/ProductHome.test.tsx
git commit -m "feat: show recent incidents on home"
```

---

## Self-Review

- Spec coverage: Tasks 1 and 2 cover SQLite schema, migrations, incident lifecycle transitions, 24-hour probe retention, and non-fatal storage errors. Task 3 covers the latest-3 Tauri command and app initialization. Tasks 4 and 5 cover the main recent incidents panel, empty/error states, and v2-only scope.
- Exclusions preserved: No full history page, filters, detailed timeline, chart, export, settings, packet capture, router action, Windows setting change, automatic recovery, or remote upload is added.
- Type consistency: Rust command returns `NetworkIncident` with camelCase serialization matching frontend `NetworkIncident`. Status values are `ongoing`, `recovering`, `resolved`; area values reuse existing diagnostic area strings.
- Verification: The final gate is `cargo test --manifest-path src-tauri/Cargo.toml`, `cargo check --manifest-path src-tauri/Cargo.toml`, `npm test`, and `npm run build`; Windows artifact validation remains required for real app data path and installer behavior.
