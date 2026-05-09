"""
analysis_runner.py — CLI entrypoint for the full analysis pipeline.

Usage
-----
    python analysis/analysis_runner.py --problem-class provable [--db results.db]
        [--output analysis/output] [--ratings subset_descriptions/medium_problems.txt]
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

# ---------------------------------------------------------------------------
# Path helpers — locate defaults relative to this script
# ---------------------------------------------------------------------------

_SCRIPT_DIR = Path(__file__).resolve().parent
_REPO_ROOT = _SCRIPT_DIR.parent


def _resolve_db(db_arg: str | None) -> Path:
    """Return the path to results.db, searching script dir then parent dir."""
    if db_arg is not None:
        return Path(db_arg).resolve()
    for candidate in [_SCRIPT_DIR / "results.db", _REPO_ROOT / "results.db"]:
        if candidate.exists():
            return candidate
    raise FileNotFoundError(
        "Could not find results.db in the script directory or its parent. "
        "Pass --db explicitly."
    )


def _resolve_ratings(ratings_arg: str | None) -> Path:
    """Return the path to the TPTP ratings file."""
    if ratings_arg is not None:
        return Path(ratings_arg).resolve()
    default = _REPO_ROOT / "subset_descriptions" / "medium_problems.txt"
    return default


def _resolve_output(output_arg: str | None) -> Path:
    """Return the base output directory."""
    if output_arg is not None:
        return Path(output_arg).resolve()
    return _SCRIPT_DIR / "output"


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Run the full benchmark analysis pipeline for a problem class."
    )
    parser.add_argument(
        "--problem-class",
        required=True,
        choices=["provable", "unprovable", "mixed", "unknown"],
        help="Problem class to analyse.",
    )
    parser.add_argument(
        "--db",
        default=None,
        metavar="PATH",
        help="Path to results.db (default: auto-detect in script dir or parent).",
    )
    parser.add_argument(
        "--output",
        default=None,
        metavar="DIR",
        help="Base output directory (default: analysis/output relative to script dir).",
    )
    parser.add_argument(
        "--ratings",
        default=None,
        metavar="PATH",
        help="Path to TPTP ratings txt file (default: subset_descriptions/medium_problems.txt).",
    )
    args = parser.parse_args()

    problem_class: str = args.problem_class

    # --- Resolve paths ---
    db_path = _resolve_db(args.db)
    ratings_path = _resolve_ratings(args.ratings)
    output_base = _resolve_output(args.output)

    print(f"Database : {db_path}")
    print(f"Ratings  : {ratings_path}")
    print(f"Output   : {output_base / problem_class}")
    print()

    # --- Ensure the script's own directory is importable ---
    script_dir_str = str(_SCRIPT_DIR)
    if script_dir_str not in sys.path:
        sys.path.insert(0, script_dir_str)

    # --- Lazy import so that path resolution errors surface first ---
    from analysis_core import (
        build_engine_results,
        load_database,
        load_tptp_ratings,
        select_runs,
    )
    from analysis_plots import (
        plot_outcome_composition,
        plot_solve_rate_by_atom_bin,
        plot_solve_rate_by_difficulty_bin,
        plot_solve_time_cdf,
    )

    # 1. Load database
    runs_df, results_df = load_database(db_path)

    # 2. Select runs
    selected_runs = select_runs(runs_df, results_df, problem_class)
    runs_summary = ", ".join(f"{e}={rid}" for e, rid in selected_runs.items())
    print(f"Selected runs: {runs_summary}")
    print()

    # 3. Load TPTP ratings
    tptp_ratings: dict[str, float] = {}
    if ratings_path.exists():
        tptp_ratings = load_tptp_ratings(ratings_path)
    else:
        print(
            f"Warning: ratings file not found at {ratings_path}; "
            "solve-rate-by-difficulty plot will be skipped.",
            file=sys.stderr,
        )

    # 4. Build engine results
    engine_results = build_engine_results(runs_df, results_df, selected_runs)

    # 5. Get shared timeout_ms from the first selected run
    first_run_id = list(selected_runs.values())[0]
    timeout_ms = int(
        runs_df.loc[runs_df["run_id"] == first_run_id, "timeout_ms"].iloc[0]
    )

    # 6. Create output directory: <output_base>/<problem_class>/
    outdir = output_base / problem_class
    outdir.mkdir(parents=True, exist_ok=True)

    # 7. Generate and save four PDFs
    plot_outcome_composition(engine_results, outdir / "outcome_composition.pdf")
    print(f"Saved: {outdir / 'outcome_composition.pdf'}")

    plot_solve_time_cdf(engine_results, timeout_ms, outdir / "solve_time_cdf.pdf")
    print(f"Saved: {outdir / 'solve_time_cdf.pdf'}")

    plot_solve_rate_by_atom_bin(engine_results, outdir / "solve_rate_by_atoms.pdf")
    print(f"Saved: {outdir / 'solve_rate_by_atoms.pdf'}")

    plot_solve_rate_by_difficulty_bin(
        engine_results, tptp_ratings, outdir / "solve_rate_by_difficulty.pdf"
    )
    print(f"Saved: {outdir / 'solve_rate_by_difficulty.pdf'}")
    print()

    # 8. Print summary table
    _print_summary(engine_results, results_df, selected_runs, problem_class)


def _print_summary(
    engine_results,
    results_df,
    selected_runs: dict[str, int],
    problem_class: str,
) -> None:
    """Print a summary table to stdout."""
    import pandas as pd

    runs_summary = ", ".join(f"{e}={rid}" for e, rid in selected_runs.items())
    print(f"=== Analysis: {problem_class} (runs: {runs_summary}) ===")
    print()

    header = f"{'Engine':<14} {'Run':>4}  {'Total':>6}  {'Solved':>6}  {'Timeout':>8}  {'max_steps':>10}  {'max_bicond':>11}  {'max_depth':>10}"
    print(header)

    for engine, run_id in selected_runs.items():
        edf = engine_results[engine_results["engine"] == engine]
        total = len(edf)
        solved = int((edf["outcome"] == "solved").sum())
        timeout = int((edf["outcome"] == "timeout").sum())
        max_steps = int((edf["outcome"] == "max_steps").sum())
        max_bicond = int((edf["outcome"] == "max_biconditionals").sum())
        max_depth = int((edf["outcome"] == "max_depth").sum())

        print(
            f"{engine:<14} {run_id:>4}  {total:>6}  {solved:>6}  {timeout:>8}  "
            f"{max_steps:>10}  {max_bicond:>11}  {max_depth:>10}"
        )


if __name__ == "__main__":
    main()
