"""
analysis_core.py — data-loading and result-processing primitives for the logic-assignment
analysis pipeline.

Public API
----------
load_database       -- read runs and results tables from a SQLite database
select_runs         -- pick one canonical run per engine for a given problem class
classify_outcome    -- map a single result row to an outcome category string
load_tptp_ratings   -- parse a TPTP problem-list file into {problem_id: rating} dict
build_engine_results -- build a flat DataFrame of (engine, problem_id, outcome, …)
"""

from __future__ import annotations

import sqlite3
from pathlib import Path

import pandas as pd


# ---------------------------------------------------------------------------
# Public functions
# ---------------------------------------------------------------------------


def load_database(db_path: str | Path) -> tuple[pd.DataFrame, pd.DataFrame]:
    """Read the ``runs`` and ``results`` tables from the SQLite database.

    Parameters
    ----------
    db_path:
        Filesystem path to the ``results.db`` SQLite file.

    Returns
    -------
    tuple[pd.DataFrame, pd.DataFrame]
        ``(runs_df, results_df)`` — one row per run and one row per result
        respectively.  Column names match the database schema exactly.
    """
    db_path = Path(db_path)
    conn = sqlite3.connect(db_path)
    try:
        runs_df = pd.read_sql_query("SELECT * FROM runs", conn)
        results_df = pd.read_sql_query("SELECT * FROM results", conn)
    finally:
        conn.close()
    return runs_df, results_df


def select_runs(
    runs_df: pd.DataFrame,
    results_df: pd.DataFrame,
    problem_class: str,
) -> dict[str, int]:
    """Select one canonical run per engine for the given *problem_class*.

    Selection rules:

    1. Filter ``runs_df`` to rows whose ``problem_class`` column equals
       *problem_class*.
    2. Per engine: pick the run with the most rows in ``results_df``; break
       ties by latest ``timestamp``.
    3. Assert that all selected runs share identical values for
       ``timeout_ms``, ``max_depth``, ``max_steps``, and
       ``max_fresh_terms_per_quantifier`` — raises ``ValueError`` if not.
    4. Assert that the count of ``biconditional_cap`` unknown results is
       equal across all selected engines (proxy for consistent
       ``max_biconditionals``) — raises ``ValueError`` if not.

    Parameters
    ----------
    runs_df:
        DataFrame returned by :func:`load_database`.
    results_df:
        DataFrame returned by :func:`load_database`.
    problem_class:
        The value to match against the ``problem_class`` column of *runs_df*
        (e.g. ``'provable'``).

    Returns
    -------
    dict[str, int]
        ``{engine_name: run_id}`` — one entry per engine.

    Raises
    ------
    ValueError
        If no runs exist for *problem_class*, or if the consistency checks
        fail.
    """
    # --- filter to the requested problem class ---
    filtered = runs_df[runs_df["problem_class"] == problem_class].copy()
    if filtered.empty:
        raise ValueError(
            f"No runs found for problem_class={problem_class!r}"
        )

    # --- count rows per run_id in results_df ---
    row_counts = (
        results_df.groupby("run_id")
        .size()
        .rename("row_count")
        .reset_index()
    )
    filtered = filtered.merge(row_counts, on="run_id", how="left")
    filtered["row_count"] = filtered["row_count"].fillna(0).astype(int)

    # --- parse timestamps for tie-breaking ---
    filtered["timestamp_dt"] = pd.to_datetime(
        filtered["timestamp"], utc=True, errors="coerce"
    )

    # --- per engine: sort descending by row_count then timestamp, keep first ---
    filtered_sorted = filtered.sort_values(
        ["engine", "row_count", "timestamp_dt"],
        ascending=[True, False, False],
    )
    selected = (
        filtered_sorted.drop_duplicates("engine", keep="first")
        .set_index("engine")
    )

    # --- consistency check: shared run parameters ---
    config_cols = [
        "timeout_ms",
        "max_depth",
        "max_steps",
        "max_fresh_terms_per_quantifier",
    ]
    for col in config_cols:
        unique_vals = selected[col].unique()
        if len(unique_vals) > 1:
            detail = dict(zip(selected.index, selected[col]))
            raise ValueError(
                f"Selected runs disagree on {col!r}: {detail}. "
                "All engines must share the same run parameters."
            )

    # --- consistency check: biconditional_cap unknown count ---
    bic_counts: dict[str, int] = {}
    for engine, row in selected.iterrows():
        run_id = int(row["run_id"])
        n = int(
            (
                (results_df["run_id"] == run_id)
                & (results_df["unknown_reason"] == "biconditional_cap")
            ).sum()
        )
        bic_counts[str(engine)] = n

    unique_bic = set(bic_counts.values())
    if len(unique_bic) > 1:
        raise ValueError(
            f"Engines disagree on biconditional_cap unknown count: {bic_counts}. "
            "This likely means max_biconditionals differs between runs."
        )

    return {str(engine): int(row["run_id"]) for engine, row in selected.iterrows()}


def classify_outcome(status: str, unknown_reason: str | None) -> str:
    """Map a single result row to an outcome category string.

    Parameters
    ----------
    status:
        The ``status`` field from the results table (e.g. ``'provable'``,
        ``'timeout'``, ``'unknown'``).
    unknown_reason:
        The ``unknown_reason`` field from the results table, or ``None`` when
        the status is not ``'unknown'``.

    Returns
    -------
    str
        One of: ``'solved'``, ``'timeout'``, ``'max_steps'``,
        ``'max_biconditionals'``, ``'max_depth'``, ``'quantifier_budget'``,
        ``'other'``.
    """
    if status in ("provable", "not_provable"):
        return "solved"
    if status == "timeout":
        return "timeout"
    if status == "unknown":
        if unknown_reason == "max_steps":
            return "max_steps"
        if unknown_reason == "biconditional_cap":
            return "max_biconditionals"
        if unknown_reason == "max_depth":
            return "max_depth"
        if unknown_reason == "quantifier_budget":
            return "quantifier_budget"
    return "other"


def load_tptp_ratings(txt_path: str | Path) -> dict[str, float]:
    """Parse a TPTP problem-list file and return a mapping of problem IDs to
    difficulty ratings.

    File format: lines beginning with ``%`` or blank lines are skipped.
    Non-comment lines are space-separated; the first column is the problem ID
    (e.g. ``SYN915+1``) and the fourth column is a float rating in ``[0, 1]``.

    Parameters
    ----------
    txt_path:
        Path to the TPTP problem list text file
        (e.g. ``subset_descriptions/medium_problems.txt``).

    Returns
    -------
    dict[str, float]
        ``{problem_id: rating}`` for every non-comment line in the file.
    """
    txt_path = Path(txt_path)
    ratings: dict[str, float] = {}
    with txt_path.open(encoding="utf-8") as fh:
        for line in fh:
            stripped = line.strip()
            if not stripped or stripped.startswith("%"):
                continue
            parts = stripped.split()
            if len(parts) < 4:
                continue
            problem_id = parts[0]
            try:
                rating = float(parts[3])
            except ValueError:
                continue
            ratings[problem_id] = rating
    return ratings


def build_engine_results(
    runs_df: pd.DataFrame,
    results_df: pd.DataFrame,
    selected_runs: dict[str, int],
) -> pd.DataFrame:
    """Build a flat DataFrame with one row per ``(engine, problem_id)`` for
    the selected runs.

    Parameters
    ----------
    runs_df:
        DataFrame returned by :func:`load_database` (used for engine lookup).
    results_df:
        DataFrame returned by :func:`load_database`.
    selected_runs:
        ``{engine_name: run_id}`` dict as returned by :func:`select_runs`.

    Returns
    -------
    pd.DataFrame
        Columns: ``engine`` (str), ``problem_id`` (str), ``outcome`` (str),
        ``elapsed_ms`` (int or NaN), ``atoms`` (int or NaN),
        ``formulae`` (int or NaN).
    """
    rows: list[dict] = []
    for engine, run_id in selected_runs.items():
        engine_results = results_df[results_df["run_id"] == run_id].copy()
        for _, result_row in engine_results.iterrows():
            outcome = classify_outcome(
                result_row["status"],
                result_row.get("unknown_reason"),
            )
            rows.append(
                {
                    "engine": engine,
                    "problem_id": result_row["problem_id"],
                    "outcome": outcome,
                    "elapsed_ms": result_row["elapsed_ms"],
                    "atoms": result_row["atoms"],
                    "formulae": result_row["formulae"],
                }
            )

    if not rows:
        return pd.DataFrame(
            columns=["engine", "problem_id", "outcome", "elapsed_ms", "atoms", "formulae"]
        )

    df = pd.DataFrame(rows)
    df["engine"] = df["engine"].astype(str)
    df["problem_id"] = df["problem_id"].astype(str)
    df["outcome"] = df["outcome"].astype(str)
    return df.reset_index(drop=True)
