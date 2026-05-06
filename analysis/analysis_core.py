from __future__ import annotations

import sqlite3
from pathlib import Path

import pandas as pd


SOLVED_STATUSES = {"provable"}
UNSOLVED_STATUSES = {"timeout", "unknown"}
OUTCOME_ORDER = ["solved", "timeout", "unknown", "other", "no_data"]
UNKNOWN_REASON_ORDER = [
    "max_depth",
    "max_steps",
    "biconditional_cap",
    "quantifier_budget",
    "other",
    "no_data",
]


def resolve_db_path() -> Path:
    base_dir = Path(__file__).resolve().parent
    candidates = [base_dir / "results.db", base_dir.parent / "results.db"]

    for candidate in candidates:
        if candidate.exists() and candidate.stat().st_size > 0:
            return candidate

    for candidate in candidates:
        if candidate.exists():
            return candidate

    raise FileNotFoundError("Could not find results.db in the script directory or its parent.")


def ensure_output_dir() -> Path:
    out_dir = Path(__file__).resolve().parent / "analysis_output"
    out_dir.mkdir(exist_ok=True)
    return out_dir


def load_database() -> tuple[pd.DataFrame, pd.DataFrame, sqlite3.Connection]:
    conn = sqlite3.connect(resolve_db_path())
    runs = pd.read_sql_query("SELECT * FROM runs", conn)
    results = pd.read_sql_query("SELECT * FROM results", conn)
    return runs, results, conn


def normalize_status(value: object) -> str | None:
    if value is None or pd.isna(value):
        return None
    return str(value).strip().lower()


def classify_status(value: object) -> str:
    status = normalize_status(value)
    if status is None:
        return "no_data"
    if status in SOLVED_STATUSES:
        return "solved"
    if status in UNSOLVED_STATUSES:
        return status
    return "other"


def classify_unknown_reason(value: object) -> str:
    reason = normalize_status(value)
    if reason is None:
        return "no_data"
    if reason in {
        "max_depth",
        "max_steps",
        "biconditional_cap",
        "quantifier_budget",
    }:
        return reason
    return "other"


def format_seconds(ms: float | int | None) -> str:
    if ms is None or pd.isna(ms):
        return "-"
    ms = float(ms)
    if ms >= 1000:
        return f"{ms / 1000:.3f}s"
    return f"{ms:.1f}ms"


def pick_representative_runs(runs: pd.DataFrame, results: pd.DataFrame) -> pd.DataFrame:
    coverage = results.groupby("run_id")["problem_id"].nunique().rename("problem_coverage")
    row_count = results.groupby("run_id").size().rename("row_count")
    selected = (
        runs.merge(coverage, on="run_id", how="left")
        .merge(row_count, on="run_id", how="left")
        .fillna({"problem_coverage": 0, "row_count": 0})
    )
    selected["timestamp_dt"] = pd.to_datetime(selected["timestamp"], utc=True, errors="coerce")
    selected = selected.sort_values(
        ["engine", "problem_coverage", "timestamp_dt", "run_id"],
        ascending=[True, False, False, False],
    )
    selected = selected.drop_duplicates("engine", keep="first").copy()
    return selected.sort_values(["engine"]).reset_index(drop=True)


def build_comparison_table(results: pd.DataFrame, selected_runs: pd.DataFrame) -> pd.DataFrame:
    selected_results = results.merge(
        selected_runs[["run_id", "engine", "label", "timestamp"]],
        on="run_id",
        how="inner",
        suffixes=("", "_run"),
    )

    engines = selected_runs["engine"].tolist()
    problem_ids = sorted(selected_results["problem_id"].unique())
    table = pd.DataFrame(index=problem_ids)

    for engine in engines:
        engine_rows = selected_results[selected_results["engine"] == engine].sort_values(["problem_id", "result_id"])
        per_problem = engine_rows.drop_duplicates("problem_id", keep="first").set_index("problem_id")
        table[f"{engine}__status"] = per_problem["status"].reindex(table.index)
        table[f"{engine}__elapsed_ms"] = per_problem["elapsed_ms"].reindex(table.index)
        table[f"{engine}__unknown_reason"] = per_problem["unknown_reason"].reindex(table.index)

    table.index.name = "problem_id"
    return table


def summarize_engine(table: pd.DataFrame, engine: str) -> dict[str, object]:
    status_col = f"{engine}__status"
    time_col = f"{engine}__elapsed_ms"

    categories = table[status_col].map(classify_status)
    counts = categories.value_counts().reindex(OUTCOME_ORDER, fill_value=0)
    solved_times = table.loc[categories == "solved", time_col].dropna().astype(float)

    return {
        "engine": engine,
        "problems": int(len(table)),
        "solved": int(counts["solved"]),
        "timeout": int(counts["timeout"]),
        "unknown": int(counts["unknown"]),
        "other": int(counts["other"]),
        "no_data": int(counts["no_data"]),
        "solve_rate": counts["solved"] / len(table) if len(table) else 0.0,
        "mean_elapsed_ms_solved": solved_times.mean() if not solved_times.empty else float("nan"),
        "median_elapsed_ms_solved": solved_times.median() if not solved_times.empty else float("nan"),
        "solved_count_for_speed": int(len(solved_times)),
    }


def compare_pair(table: pd.DataFrame, engine_a: str, engine_b: str) -> tuple[dict[str, object], pd.DataFrame]:
    a_status = table[f"{engine_a}__status"].map(classify_status)
    b_status = table[f"{engine_b}__status"].map(classify_status)
    a_time = table[f"{engine_a}__elapsed_ms"].astype(float)
    b_time = table[f"{engine_b}__elapsed_ms"].astype(float)

    rows = []
    for problem_id in table.index:
        ca = a_status.loc[problem_id]
        cb = b_status.loc[problem_id]
        ta = a_time.loc[problem_id]
        tb = b_time.loc[problem_id]

        if ca == "no_data" or cb == "no_data":
            outcome = "no_data"
            winner = None
        elif ca == "solved" and cb == "solved":
            if ta < tb:
                outcome = "a_wins"
                winner = engine_a
            elif tb < ta:
                outcome = "b_wins"
                winner = engine_b
            else:
                outcome = "tie"
                winner = None
        elif ca == "solved" and cb != "solved":
            outcome = "a_wins"
            winner = engine_a
        elif cb == "solved" and ca != "solved":
            outcome = "b_wins"
            winner = engine_b
        else:
            outcome = "tie"
            winner = None

        rows.append(
            {
                "problem_id": problem_id,
                f"{engine_a}_status": ca,
                f"{engine_b}_status": cb,
                f"{engine_a}_elapsed_ms": ta,
                f"{engine_b}_elapsed_ms": tb,
                "outcome": outcome,
                "winner": winner,
                "delta_ms_b_minus_a": tb - ta if pd.notna(ta) and pd.notna(tb) else float("nan"),
                "ratio_b_over_a": tb / ta if pd.notna(ta) and pd.notna(tb) and ta > 0 else float("nan"),
            }
        )

    pair_df = pd.DataFrame(rows).set_index("problem_id")

    comparable = pair_df[pair_df["outcome"] != "no_data"]
    shared_solved = pair_df[(pair_df[f"{engine_a}_status"] == "solved") & (pair_df[f"{engine_b}_status"] == "solved")]

    metrics = {
        "engine_a": engine_a,
        "engine_b": engine_b,
        "problems": int(len(pair_df)),
        "comparable": int(len(comparable)),
        "a_wins": int((pair_df["outcome"] == "a_wins").sum()),
        "b_wins": int((pair_df["outcome"] == "b_wins").sum()),
        "ties": int((pair_df["outcome"] == "tie").sum()),
        "no_data": int((pair_df["outcome"] == "no_data").sum()),
        "shared_solved": int(len(shared_solved)),
        "shared_solved_a_mean": shared_solved[f"{engine_a}_elapsed_ms"].mean() if not shared_solved.empty else float("nan"),
        "shared_solved_b_mean": shared_solved[f"{engine_b}_elapsed_ms"].mean() if not shared_solved.empty else float("nan"),
        "shared_solved_delta_mean": shared_solved["delta_ms_b_minus_a"].mean() if not shared_solved.empty else float("nan"),
        "shared_solved_ratio_median": shared_solved["ratio_b_over_a"].median() if not shared_solved.empty else float("nan"),
    }

    return metrics, pair_df
