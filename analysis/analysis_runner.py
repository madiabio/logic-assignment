from __future__ import annotations

import math

import pandas as pd

from analysis_core import (
    build_comparison_table,
    compare_pair,
    classify_status,
    classify_unknown_reason,
    ensure_output_dir,
    load_database,
    pick_representative_runs,
    resolve_db_path,
    UNKNOWN_REASON_ORDER,
    summarize_engine,
)
from analysis_plots import save_bar_chart, save_boxplot, save_heatmap, save_histogram
from analysis_report import write_report


def main() -> None:
    out_dir = ensure_output_dir()
    data_path = resolve_db_path()
    runs, results, conn = load_database()

    try:
        selected_runs = pick_representative_runs(runs, results)
        comparison_table = build_comparison_table(results, selected_runs)

        engine_summary = pd.DataFrame([summarize_engine(comparison_table, engine) for engine in selected_runs["engine"]])

        unknown_reason_rows = []
        for engine in selected_runs["engine"]:
            unknown_mask = comparison_table[f"{engine}__status"].map(classify_status) == "unknown"
            reason_counts = (
                comparison_table.loc[unknown_mask, f"{engine}__unknown_reason"]
                .map(classify_unknown_reason)
                .value_counts()
                .reindex(UNKNOWN_REASON_ORDER, fill_value=0)
            )
            unknown_reason_rows.append({"engine": engine, **reason_counts.to_dict()})
        unknown_reason_summary = pd.DataFrame(unknown_reason_rows)

        pairwise_rows = []
        pair_tables: dict[tuple[str, str], pd.DataFrame] = {}
        engines = selected_runs["engine"].tolist()
        for i, engine_a in enumerate(engines):
            for engine_b in engines[i + 1 :]:
                metrics, pair_df = compare_pair(comparison_table, engine_a, engine_b)
                pairwise_rows.append(metrics)
                pair_tables[(engine_a, engine_b)] = pair_df

        pairwise_summary = pd.DataFrame(pairwise_rows)

        selected_runs_out = selected_runs[
            ["engine", "run_id", "label", "timestamp", "problem_coverage", "row_count"]
        ].rename(columns={"problem_coverage": "problems"})
        selected_runs_out.to_csv(out_dir / "selected_runs.csv", index=False)
        engine_summary.to_csv(out_dir / "engine_summary.csv", index=False)
        unknown_reason_summary.to_csv(out_dir / "unknown_reason_summary.csv", index=False)
        pairwise_summary.to_csv(out_dir / "pairwise_summary.csv", index=False)
        comparison_table.reset_index().to_csv(out_dir / "problem_comparison.csv", index=False)

        plot_order = selected_runs["engine"].tolist()
        engine_summary_plot = engine_summary.set_index("engine").loc[plot_order]

        save_bar_chart(
            (engine_summary_plot["solve_rate"] * 100).to_frame("solve_rate_pct"),
            out_dir / "solve_rate.png",
            "Solved Rate by Engine",
            "Solved rate (%)",
            stacked=False,
        )

        save_bar_chart(
            engine_summary_plot[["solved", "timeout", "unknown", "other", "no_data"]],
            out_dir / "outcome_composition.png",
            "Outcome Composition by Engine",
            "Problems",
            stacked=True,
        )

        if unknown_reason_summary[UNKNOWN_REASON_ORDER[:-1]].to_numpy().sum() > 0:
            save_bar_chart(
                unknown_reason_summary.set_index("engine")[UNKNOWN_REASON_ORDER[:-1]],
                out_dir / "unknown_reason_composition.png",
                "Unknown Reasons by Engine",
                "Unknown cases",
                stacked=True,
            )

        solved_times = pd.DataFrame(
            {
                engine: comparison_table.loc[
                    comparison_table[f"{engine}__status"].map(lambda value: classify_status(value) == "solved"),
                    f"{engine}__elapsed_ms",
                ]
                .astype(float)
                .rename(engine)
                for engine in plot_order
            }
        )
        save_boxplot(
            solved_times,
            out_dir / "elapsed_distribution.png",
            "Solved-Case Elapsed Time Distribution",
            "Elapsed time (ms, log scale)",
        )

        if not pairwise_summary.empty:
            pairwise_matrix = pd.DataFrame(index=plot_order, columns=plot_order, dtype=float)
            pairwise_labels = pd.DataFrame(index=plot_order, columns=plot_order, dtype=object)
            for (engine_a, engine_b), pair_df in pair_tables.items():
                comparable = int((pair_df["outcome"] != "no_data").sum())
                a_wins = int((pair_df["outcome"] == "a_wins").sum())
                b_wins = int((pair_df["outcome"] == "b_wins").sum())
                share_ab = a_wins / comparable if comparable else float("nan")
                share_ba = b_wins / comparable if comparable else float("nan")
                pairwise_matrix.loc[engine_a, engine_b] = share_ab
                pairwise_matrix.loc[engine_b, engine_a] = share_ba
                pairwise_labels.loc[engine_a, engine_b] = f"{a_wins}/{comparable}"
                pairwise_labels.loc[engine_b, engine_a] = f"{b_wins}/{comparable}"
            for engine in plot_order:
                pairwise_matrix.loc[engine, engine] = 0.5
                pairwise_labels.loc[engine, engine] = "self"
            save_heatmap(
                pairwise_matrix,
                out_dir / "pairwise_win_rate.png",
                "Pairwise Win Share",
                annotation=pairwise_labels,
            )

            for (engine_a, engine_b), pair_df in pair_tables.items():
                shared = pair_df[
                    (pair_df[f"{engine_a}_status"] == "solved")
                    & (pair_df[f"{engine_b}_status"] == "solved")
                    & pair_df["ratio_b_over_a"].notna()
                ]
                if shared.empty:
                    continue
                save_histogram(
                    pd.Series([math.log2(x) for x in shared["ratio_b_over_a"].to_numpy() if x > 0]),
                    out_dir / f"paired_speed_ratio_{engine_a}_vs_{engine_b}.png",
                    f"Paired Speed Ratio: {engine_b} vs {engine_a}",
                    "log2(elapsed_b / elapsed_a)",
                )

        report_path = write_report(
            out_dir,
            selected_runs,
            engine_summary,
            unknown_reason_summary,
            pairwise_summary,
            comparison_table,
            pair_tables,
        )

        print(f"Using database: {data_path}")
        print(f"Output directory: {out_dir}")
        print(f"Report written to: {report_path}")
        print()
        print("Selected runs:")
        print(selected_runs_out.to_string(index=False))
        print()
        print("Engine summary:")
        print(
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
            ].to_string(index=False)
        )
        if not unknown_reason_summary.empty:
            print()
            print("Unknown reason summary:")
            print(unknown_reason_summary.to_string(index=False))
        if not pairwise_summary.empty:
            print()
            print("Pairwise summary:")
            print(pairwise_summary.to_string(index=False))
    finally:
        conn.close()
