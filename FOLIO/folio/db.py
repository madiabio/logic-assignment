from __future__ import annotations

import pathlib
import sqlite3
from contextlib import closing
from dataclasses import dataclass


@dataclass(frozen=True)
class ResultDetail:
    status: str
    elapsed_ms: int
    unknown_reason: str | None


@dataclass(frozen=True)
class RunConstraints:
    label: str
    engine: str
    timeout_ms: int
    max_depth: int
    max_steps: int
    max_fresh_terms_per_quantifier: int
    problem_class: str


_MISSING_DETAIL = ResultDetail(status="missing", elapsed_ms=0, unknown_reason=None)


def latest_run_id(db_path: pathlib.Path) -> int | None:
    if not db_path.exists():
        return None
    with closing(sqlite3.connect(db_path)) as conn:
        row = conn.execute("SELECT MAX(run_id) FROM runs").fetchone()
        return int(row[0]) if row and row[0] is not None else None


def result_details(db_path: pathlib.Path, run_id: int | None) -> dict[str, ResultDetail]:
    if run_id is None or not db_path.exists():
        return {}
    with closing(sqlite3.connect(db_path)) as conn:
        rows = conn.execute(
            "SELECT problem_id, status, elapsed_ms, unknown_reason FROM results WHERE run_id = ?",
            (run_id,),
        ).fetchall()
    return {
        problem_id: ResultDetail(status=status, elapsed_ms=elapsed_ms, unknown_reason=unknown_reason)
        for problem_id, status, elapsed_ms, unknown_reason in rows
    }


def fetch_run_constraints(db_path: pathlib.Path, run_id: int | None) -> RunConstraints | None:
    if run_id is None or not db_path.exists():
        return None
    with closing(sqlite3.connect(db_path)) as conn:
        row = conn.execute(
            "SELECT label, engine, timeout_ms, max_depth, max_steps, max_fresh_terms_per_quantifier, problem_class "
            "FROM runs WHERE run_id = ?",
            (run_id,),
        ).fetchone()
    if row is None:
        return None
    label, engine, timeout_ms, max_depth, max_steps, mftpq, problem_class = row
    return RunConstraints(
        label=label,
        engine=engine,
        timeout_ms=int(timeout_ms),
        max_depth=int(max_depth),
        max_steps=int(max_steps),
        max_fresh_terms_per_quantifier=int(mftpq),
        problem_class=problem_class,
    )


def detail_for(details: dict[str, ResultDetail], path: pathlib.Path) -> ResultDetail:
    return details.get(path.stem, _MISSING_DETAIL)
