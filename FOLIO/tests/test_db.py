import pathlib
import sqlite3
import tempfile
import unittest

from folio.db import (
    ResultDetail,
    RunConstraints,
    _MISSING_DETAIL,
    detail_for,
    fetch_run_constraints,
    latest_run_id,
    result_details,
)

_SCHEMA = """
CREATE TABLE runs (
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
CREATE TABLE results (
    result_id   INTEGER PRIMARY KEY AUTOINCREMENT,
    run_id      INTEGER NOT NULL REFERENCES runs(run_id),
    problem_id  TEXT    NOT NULL,
    path        TEXT    NOT NULL,
    status      TEXT    NOT NULL,
    elapsed_ms  INTEGER NOT NULL,
    formulae    INTEGER,
    atoms       INTEGER,
    unknown_reason TEXT
);
"""

_SAMPLE_RUN = (
    "test-run",
    "2026-01-01T00:00:00Z",
    "priority-id",
    1000,
    128,
    50000,
    1,
    "mixed",
)


def _make_db(path: pathlib.Path, results: list[dict] | None = None) -> int:
    """Create a DB at path with schema, one run, and optional results. Returns run_id."""
    conn = sqlite3.connect(path)
    try:
        conn.executescript(_SCHEMA)
        conn.execute(
            "INSERT INTO runs (label, timestamp, engine, timeout_ms, max_depth, max_steps, "
            "max_fresh_terms_per_quantifier, problem_class) VALUES (?,?,?,?,?,?,?,?)",
            _SAMPLE_RUN,
        )
        run_id = conn.execute("SELECT last_insert_rowid()").fetchone()[0]
        for r in results or []:
            conn.execute(
                "INSERT INTO results (run_id, problem_id, path, status, elapsed_ms, unknown_reason) "
                "VALUES (?,?,?,?,?,?)",
                (run_id, r["problem_id"], r.get("path", "/tmp/x.p"), r["status"],
                 r.get("elapsed_ms", 0), r.get("unknown_reason")),
            )
        conn.commit()
    finally:
        conn.close()
    return run_id


class TestLatestRunId(unittest.TestCase):
    def setUp(self):
        self._tmpdir = tempfile.TemporaryDirectory()
        self._dir = pathlib.Path(self._tmpdir.name)

    def tearDown(self):
        self._tmpdir.cleanup()

    def test_missing_db_returns_none(self):
        self.assertIsNone(latest_run_id(self._dir / "nonexistent.db"))

    def test_empty_db_returns_none(self):
        db = self._dir / "empty.db"
        conn = sqlite3.connect(db)
        conn.executescript(_SCHEMA)
        conn.close()
        self.assertIsNone(latest_run_id(db))

    def test_returns_max_run_id(self):
        db = self._dir / "two_runs.db"
        _make_db(db)
        conn = sqlite3.connect(db)
        conn.execute(
            "INSERT INTO runs (label, timestamp, engine, timeout_ms, max_depth, max_steps, "
            "max_fresh_terms_per_quantifier, problem_class) VALUES (?,?,?,?,?,?,?,?)",
            _SAMPLE_RUN,
        )
        conn.commit()
        conn.close()
        self.assertEqual(latest_run_id(db), 2)


class TestResultDetails(unittest.TestCase):
    def setUp(self):
        self._tmpdir = tempfile.TemporaryDirectory()
        self._dir = pathlib.Path(self._tmpdir.name)

    def tearDown(self):
        self._tmpdir.cleanup()

    def test_empty_when_no_results(self):
        db = self._dir / "empty.db"
        run_id = _make_db(db)
        self.assertEqual(result_details(db, run_id), {})

    def test_empty_when_run_id_is_none(self):
        self.assertEqual(result_details(self._dir / "nope.db", None), {})

    def test_fetches_status_elapsed_and_unknown_reason(self):
        db = self._dir / "with_results.db"
        run_id = _make_db(db, [
            {"problem_id": "prob1", "status": "unknown", "elapsed_ms": 42, "unknown_reason": "max_depth"},
        ])
        details = result_details(db, run_id)
        self.assertIn("prob1", details)
        d = details["prob1"]
        self.assertEqual(d.status, "unknown")
        self.assertEqual(d.elapsed_ms, 42)
        self.assertEqual(d.unknown_reason, "max_depth")

    def test_unknown_reason_is_none_for_provable(self):
        db = self._dir / "provable.db"
        run_id = _make_db(db, [
            {"problem_id": "prov1", "status": "provable", "elapsed_ms": 10},
        ])
        details = result_details(db, run_id)
        self.assertIsNone(details["prov1"].unknown_reason)


class TestFetchRunConstraints(unittest.TestCase):
    def setUp(self):
        self._tmpdir = tempfile.TemporaryDirectory()
        self._dir = pathlib.Path(self._tmpdir.name)

    def tearDown(self):
        self._tmpdir.cleanup()

    def test_returns_none_for_missing_db(self):
        self.assertIsNone(fetch_run_constraints(self._dir / "nope.db", 1))

    def test_returns_none_for_missing_run_id(self):
        db = self._dir / "test.db"
        _make_db(db)
        self.assertIsNone(fetch_run_constraints(db, 999))

    def test_all_fields_round_trip(self):
        db = self._dir / "test.db"
        run_id = _make_db(db)
        c = fetch_run_constraints(db, run_id)
        self.assertIsInstance(c, RunConstraints)
        self.assertEqual(c.label, "test-run")
        self.assertEqual(c.engine, "priority-id")
        self.assertEqual(c.timeout_ms, 1000)
        self.assertEqual(c.max_depth, 128)
        self.assertEqual(c.max_steps, 50000)
        self.assertEqual(c.max_fresh_terms_per_quantifier, 1)
        self.assertEqual(c.problem_class, "mixed")


class TestDetailFor(unittest.TestCase):
    def test_missing_key_returns_missing_detail(self):
        result = detail_for({}, pathlib.Path("some__problem__entails.p"))
        self.assertEqual(result, _MISSING_DETAIL)
        self.assertEqual(result.status, "missing")

    def test_known_key_returns_correct_detail(self):
        d = ResultDetail(status="provable", elapsed_ms=123, unknown_reason=None)
        result = detail_for({"my_problem": d}, pathlib.Path("my_problem.p"))
        self.assertEqual(result, d)


if __name__ == "__main__":
    unittest.main()
