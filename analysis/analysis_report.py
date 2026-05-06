from __future__ import annotations

from pathlib import Path

import pandas as pd

from analysis_core import UNKNOWN_REASON_ORDER, format_seconds


def format_ratio(value: float | int | None) -> str:
    if value is None or pd.isna(value):
        return "-"
    return f"{float(value):.3f}x"


def df_to_markdown(df: pd.DataFrame, index: bool = False) -> str:
    frame = df.copy()
    if index:
        frame = frame.reset_index()

    if frame.empty:
        return "_No rows._"

    headers = list(frame.columns)
    rows = [headers]
    for _, row in frame.iterrows():
        rows.append([str(item).replace("|", "\\|") for item in row.tolist()])

    widths = [max(len(str(row[i])) for row in rows) for i in range(len(headers))]

    def render_row(values: list[object]) -> str:
        cells = [str(v).replace("|", "\\|").ljust(widths[i]) for i, v in enumerate(values)]
        return "| " + " | ".join(cells) + " |"

    lines = [render_row(rows[0])]
    lines.append("| " + " | ".join("-" * w for w in widths) + " |")
    for values in rows[1:]:
        lines.append(render_row(values))
    return "\n".join(lines)


def write_report(
    out_dir: Path,
    selected_runs: pd.DataFrame,
    engine_summary: pd.DataFrame,
    unknown_reason_summary: pd.DataFrame,
    pairwise_summary: pd.DataFrame,
    comparison_table: pd.DataFrame,
    pair_tables: dict[tuple[str, str], pd.DataFrame],
) -> Path:
    report_path = out_dir / "analysis_report.md"
    lines: list[str] = []
    lines.append("# Engine Comparison Report")
    lines.append("")
    lines.append("## Selected Runs")
    lines.append("")
    lines.append(
        df_to_markdown(
            selected_runs[["engine", "run_id", "label", "timestamp", "problem_coverage", "row_count"]]
            .rename(columns={"problem_coverage": "problems"})
            .sort_values("engine")
            .reset_index(drop=True)
        )
    )
    lines.append("")
    lines.append("## Engine Summary")
    lines.append("")
    lines.append(
        df_to_markdown(
            engine_summary[
                [
                    "engine",
                    "problems",
                    "solved",
                    "timeout",
                    "unknown",
                    "other",
                    "no_data",
                    "solve_rate",
                    "mean_elapsed_ms_solved",
                    "median_elapsed_ms_solved",
                ]
            ].assign(
                solve_rate=lambda df: (df["solve_rate"] * 100).map(lambda x: f"{x:.1f}%"),
                mean_elapsed_ms_solved=lambda df: df["mean_elapsed_ms_solved"].map(format_seconds),
                median_elapsed_ms_solved=lambda df: df["median_elapsed_ms_solved"].map(format_seconds),
            )
        )
    )
    lines.append("")
    lines.append("## Unknown Reasons")
    lines.append("")
    lines.append(
        "These counts split the `unknown` bucket into the concrete stopping reasons recorded by the prover."
    )
    lines.append("")
    if not unknown_reason_summary.empty:
        reason_cols = [col for col in UNKNOWN_REASON_ORDER if col != "no_data"]
        lines.append(
            df_to_markdown(
                unknown_reason_summary[["engine", *reason_cols]].sort_values("engine").reset_index(drop=True)
            )
        )
    else:
        lines.append("_No unknown results were recorded._")
    lines.append("")
    lines.append("## Pairwise Summary")
    lines.append("")
    pairwise_md = pairwise_summary.copy()
    pairwise_md["a_win_share"] = pairwise_md.apply(
        lambda row: f"{(row['a_wins'] / row['comparable'] * 100):.1f}%" if row["comparable"] else "n/a",
        axis=1,
    )
    pairwise_md["b_win_share"] = pairwise_md.apply(
        lambda row: f"{(row['b_wins'] / row['comparable'] * 100):.1f}%" if row["comparable"] else "n/a",
        axis=1,
    )
    pairwise_md["shared_solved_a_mean"] = pairwise_md["shared_solved_a_mean"].map(format_seconds)
    pairwise_md["shared_solved_b_mean"] = pairwise_md["shared_solved_b_mean"].map(format_seconds)
    pairwise_md["shared_solved_delta_mean"] = pairwise_md["shared_solved_delta_mean"].map(format_seconds)
    lines.append(
        df_to_markdown(
            pairwise_md[
                [
                    "engine_a",
                    "engine_b",
                    "problems",
                    "comparable",
                    "a_wins",
                    "b_wins",
                    "ties",
                    "no_data",
                    "shared_solved",
                    "a_win_share",
                    "b_win_share",
                    "shared_solved_a_mean",
                    "shared_solved_b_mean",
                    "shared_solved_delta_mean",
                ]
            ]
        )
    )
    lines.append("")
    lines.append("## Problem-Level Comparison")
    lines.append("")
    lines.append(df_to_markdown(comparison_table.reset_index().head(20)))
    lines.append("")
    lines.append("## Top Shared-Solved Speed Differences")
    lines.append("")
    for (engine_a, engine_b), pair_df in pair_tables.items():
        shared = pair_df[
            (pair_df[f"{engine_a}_status"] == "solved")
            & (pair_df[f"{engine_b}_status"] == "solved")
            & pair_df["ratio_b_over_a"].notna()
        ].copy()
        lines.append(f"### {engine_a} vs {engine_b}")
        if shared.empty:
            lines.append("_No shared solved problems._")
            lines.append("")
            continue

        fastest_for_a = shared.sort_values("ratio_b_over_a").head(10).reset_index()
        fastest_for_b = shared.sort_values("ratio_b_over_a", ascending=False).head(10).reset_index()

        display_a = fastest_for_a[
            [
                "problem_id",
                f"{engine_a}_elapsed_ms",
                f"{engine_b}_elapsed_ms",
                "ratio_b_over_a",
                "delta_ms_b_minus_a",
            ]
        ].copy()
        display_b = fastest_for_b[
            [
                "problem_id",
                f"{engine_a}_elapsed_ms",
                f"{engine_b}_elapsed_ms",
                "ratio_b_over_a",
                "delta_ms_b_minus_a",
            ]
        ].copy()

        for frame in (display_a, display_b):
            frame[f"{engine_a}_elapsed_ms"] = frame[f"{engine_a}_elapsed_ms"].map(format_seconds)
            frame[f"{engine_b}_elapsed_ms"] = frame[f"{engine_b}_elapsed_ms"].map(format_seconds)
            frame["ratio_b_over_a"] = frame["ratio_b_over_a"].map(format_ratio)
            frame["delta_ms_b_minus_a"] = frame["delta_ms_b_minus_a"].map(format_seconds)

        lines.append("Fastest for engine_a relative to engine_b:")
        lines.append(df_to_markdown(display_a))
        lines.append("")
        lines.append("Fastest for engine_b relative to engine_a:")
        lines.append(df_to_markdown(display_b))
        lines.append("")

    report_path.write_text("\n".join(lines), encoding="utf-8")
    return report_path
