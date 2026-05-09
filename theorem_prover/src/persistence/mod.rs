//! Persistence layer for theorem prover proof runs.
//!
//! Owns all SQLite database logic for storing and querying proof run metadata
//! and per-problem results. No CLI concerns belong here.

use rusqlite::{Connection, params};
use std::collections::HashMap;
use std::path::Path;

/// Metadata about a proof run stored before the batch starts.
pub struct RunRecord {
    /// Human-readable label for the run.
    pub label: String,
    /// ISO 8601 timestamp when the run was initiated.
    pub timestamp: String,
    /// Name of the search engine used (e.g. "naive", "iterative_deepening").
    pub engine: String,
    /// Per-problem timeout in milliseconds.
    pub timeout_ms: u64,
    /// Maximum proof search depth.
    pub max_depth: u32,
    /// Maximum number of proof steps.
    pub max_steps: u64,
    /// Maximum number of fresh terms introduced per quantifier.
    pub max_fresh_terms_per_quantifier: u32,
    /// Expected difficulty class of the problem set (e.g. "provable", "mixed").
    pub problem_class: String,
}

/// Result for a single problem in a batch.
pub struct ResultRecord {
    /// Unique identifier for the problem (e.g. its TPTP name).
    pub problem_id: String,
    /// Filesystem path to the problem file.
    pub path: String,
    /// Outcome of the proof attempt: "provable", "timeout", or "unknown".
    pub status: String,
    /// Wall-clock time taken for this problem in milliseconds.
    pub elapsed_ms: u128,
    /// Number of formulae in the problem, if known.
    pub formulae: Option<i64>,
    /// Number of atoms in the problem, if known.
    pub atoms: Option<i64>,
    /// Reason the result is unknown; NULL unless status is "unknown".
    pub unknown_reason: Option<String>,
}

/// Open (or create) a SQLite database at the given path.
pub fn open_db(path: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    Ok(conn)
}

/// Create the `runs` and `results` tables if they do not already exist,
/// and migrate any pre-existing `runs` table to include `problem_class`.
pub fn ensure_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS runs (
            run_id      INTEGER PRIMARY KEY AUTOINCREMENT,
            label       TEXT    NOT NULL,
            timestamp   TEXT    NOT NULL,
            engine      TEXT    NOT NULL,
            timeout_ms              INTEGER NOT NULL,
            max_depth               INTEGER NOT NULL,
            max_steps               INTEGER NOT NULL,
            max_fresh_terms_per_quantifier INTEGER NOT NULL,
            problem_class           TEXT    NOT NULL DEFAULT 'unknown'
        );

        CREATE TABLE IF NOT EXISTS results (
            result_id   INTEGER PRIMARY KEY AUTOINCREMENT,
            run_id      INTEGER NOT NULL REFERENCES runs(run_id),
            problem_id  TEXT    NOT NULL,
            path        TEXT    NOT NULL,
            status      TEXT    NOT NULL CHECK(status IN ('provable', 'not_provable', 'timeout', 'unknown', 'cancelled', 'not_implemented', 'error')),
            elapsed_ms  INTEGER NOT NULL,
            formulae    INTEGER,
            atoms       INTEGER,
            unknown_reason TEXT
        );",
    )?;

    // Migrate pre-existing `runs` tables that predate the problem_class column.
    let has_column: bool = conn
        .prepare("PRAGMA table_info(runs)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .any(|name| name.as_deref() == Ok("problem_class"));
    if !has_column {
        conn.execute_batch(
            "ALTER TABLE runs ADD COLUMN problem_class TEXT NOT NULL DEFAULT 'unknown';",
        )?;
    }

    Ok(())
}

/// Insert a run record and return its run_id.
pub fn insert_run(conn: &Connection, run: &RunRecord) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO runs (label, timestamp, engine, timeout_ms, max_depth, max_steps, max_fresh_terms_per_quantifier, problem_class)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            run.label,
            run.timestamp,
            run.engine,
            run.timeout_ms as i64,
            run.max_depth,
            run.max_steps as i64,
            run.max_fresh_terms_per_quantifier,
            run.problem_class,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Insert a single problem result. Each call commits immediately (auto-commit).
pub fn insert_result(conn: &Connection, run_id: i64, result: &ResultRecord) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO results (run_id, problem_id, path, status, elapsed_ms, formulae, atoms, unknown_reason)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            run_id,
            result.problem_id,
            result.path,
            result.status,
            result.elapsed_ms as i64,
            result.formulae,
            result.atoms,
            result.unknown_reason,
        ],
    )?;
    Ok(())
}

/// Query aggregated status counts for a given run_id.
///
/// Returns a map from status string (e.g. "provable", "timeout") to the number
/// of results with that status in the given run.
pub fn query_run_summary(conn: &Connection, run_id: i64) -> rusqlite::Result<HashMap<String, u64>> {
    let mut stmt = conn.prepare(
        "SELECT status, COUNT(*) FROM results WHERE run_id = ?1 GROUP BY status",
    )?;
    let rows = stmt.query_map(params![run_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
    })?;

    let mut map = HashMap::new();
    for row in rows {
        let (status, count) = row?;
        map.insert(status, count as u64);
    }
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn in_memory() -> Connection {
        Connection::open_in_memory().unwrap()
    }

    fn sample_run() -> RunRecord {
        RunRecord {
            label: "test-run".to_string(),
            timestamp: "2026-05-06T00:00:00Z".to_string(),
            engine: "naive".to_string(),
            timeout_ms: 5000,
            max_depth: 10,
            max_steps: 1000,
            max_fresh_terms_per_quantifier: 3,
            problem_class: "mixed".to_string(),
        }
    }

    fn sample_result(status: &str) -> ResultRecord {
        ResultRecord {
            problem_id: "prob1".to_string(),
            path: "/tmp/prob1.p".to_string(),
            status: status.to_string(),
            elapsed_ms: 42,
            formulae: Some(5),
            atoms: Some(8),
            unknown_reason: if status == "unknown" {
                Some("depth limit".to_string())
            } else {
                None
            },
        }
    }

    #[test]
    fn ensure_schema_creates_both_tables() {
        let conn = in_memory();
        ensure_schema(&conn).unwrap();

        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap();
        let names: Vec<String> = stmt
            .query_map([], |row| row.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        assert!(names.contains(&"results".to_string()));
        assert!(names.contains(&"runs".to_string()));
    }

    #[test]
    fn ensure_schema_is_idempotent() {
        let conn = in_memory();
        ensure_schema(&conn).unwrap();
        ensure_schema(&conn).unwrap(); // must not error
    }

    #[test]
    fn insert_run_returns_incrementing_ids() {
        let conn = in_memory();
        ensure_schema(&conn).unwrap();

        let id1 = insert_run(&conn, &sample_run()).unwrap();
        let id2 = insert_run(&conn, &sample_run()).unwrap();

        assert!(id1 > 0);
        assert!(id2 > id1);
    }

    #[test]
    fn insert_run_stores_all_fields() {
        let conn = in_memory();
        ensure_schema(&conn).unwrap();

        let run = RunRecord {
            label: "my-label".to_string(),
            timestamp: "2026-01-01T12:00:00Z".to_string(),
            engine: "iterative_deepening".to_string(),
            timeout_ms: 9999,
            max_depth: 20,
            max_steps: 5000,
            max_fresh_terms_per_quantifier: 7,
            problem_class: "provable".to_string(),
        };
        let id = insert_run(&conn, &run).unwrap();

        let (label, timestamp, engine, timeout_ms, max_depth, max_steps, mft, problem_class): (
            String, String, String, i64, i64, i64, i64, String,
        ) = conn
            .query_row(
                "SELECT label, timestamp, engine, timeout_ms, max_depth, max_steps, max_fresh_terms_per_quantifier, problem_class FROM runs WHERE run_id = ?1",
                params![id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?, row.get(7)?)),
            )
            .unwrap();

        assert_eq!(label, "my-label");
        assert_eq!(timestamp, "2026-01-01T12:00:00Z");
        assert_eq!(engine, "iterative_deepening");
        assert_eq!(timeout_ms, 9999);
        assert_eq!(max_depth, 20);
        assert_eq!(max_steps, 5000);
        assert_eq!(mft, 7);
        assert_eq!(problem_class, "provable");
    }

    #[test]
    fn insert_result_stores_all_fields() {
        let conn = in_memory();
        ensure_schema(&conn).unwrap();
        let run_id = insert_run(&conn, &sample_run()).unwrap();

        let result = ResultRecord {
            problem_id: "p42".to_string(),
            path: "/data/p42.p".to_string(),
            status: "unknown".to_string(),
            elapsed_ms: 1234,
            formulae: Some(11),
            atoms: Some(22),
            unknown_reason: Some("step limit".to_string()),
        };
        insert_result(&conn, run_id, &result).unwrap();

        let (problem_id, path, status, elapsed_ms, formulae, atoms, unknown_reason): (
            String, String, String, i64, Option<i64>, Option<i64>, Option<String>,
        ) = conn
            .query_row(
                "SELECT problem_id, path, status, elapsed_ms, formulae, atoms, unknown_reason FROM results WHERE run_id = ?1",
                params![run_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?)),
            )
            .unwrap();

        assert_eq!(problem_id, "p42");
        assert_eq!(path, "/data/p42.p");
        assert_eq!(status, "unknown");
        assert_eq!(elapsed_ms, 1234);
        assert_eq!(formulae, Some(11));
        assert_eq!(atoms, Some(22));
        assert_eq!(unknown_reason, Some("step limit".to_string()));
    }

    #[test]
    fn insert_result_null_unknown_reason() {
        let conn = in_memory();
        ensure_schema(&conn).unwrap();
        let run_id = insert_run(&conn, &sample_run()).unwrap();

        let result = ResultRecord {
            problem_id: "prov1".to_string(),
            path: "/data/prov1.p".to_string(),
            status: "provable".to_string(),
            elapsed_ms: 10,
            formulae: None,
            atoms: None,
            unknown_reason: None,
        };
        insert_result(&conn, run_id, &result).unwrap();

        let unknown_reason: Option<String> = conn
            .query_row(
                "SELECT unknown_reason FROM results WHERE run_id = ?1",
                params![run_id],
                |row| row.get(0),
            )
            .unwrap();

        assert!(unknown_reason.is_none());
    }

    #[test]
    fn query_run_summary_counts_by_status() {
        let conn = in_memory();
        ensure_schema(&conn).unwrap();
        let run_id = insert_run(&conn, &sample_run()).unwrap();

        for _ in 0..3 {
            insert_result(&conn, run_id, &sample_result("provable")).unwrap();
        }
        for _ in 0..2 {
            insert_result(&conn, run_id, &sample_result("timeout")).unwrap();
        }

        let summary = query_run_summary(&conn, run_id).unwrap();

        assert_eq!(summary.get("provable"), Some(&3u64));
        assert_eq!(summary.get("timeout"), Some(&2u64));
        assert_eq!(summary.len(), 2);
    }

    #[test]
    fn insert_result_rejects_invalid_status() {
        let conn = in_memory();
        ensure_schema(&conn).unwrap();
        let run_id = insert_run(&conn, &sample_run()).unwrap();

        let mut invalid_result = sample_result("provable");
        invalid_result.status = "invalid_status".to_string();

        let result = insert_result(&conn, run_id, &invalid_result);
        assert!(result.is_err(), "INSERT should fail for invalid status");
    }

    #[test]
    fn open_db_enables_wal_journal_mode() {
        let path = std::env::temp_dir().join("tp_wal_test.db");
        let conn = open_db(&path).unwrap();
        let mode: String = conn
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .unwrap();
        let _ = std::fs::remove_file(&path);
        assert_eq!(mode, "wal");
    }
}
