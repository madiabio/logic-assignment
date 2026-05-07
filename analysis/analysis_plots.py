"""
Publication-quality plot functions for the theorem-prover benchmark analysis.

Each function produces a single PDF figure suitable for direct inclusion in a
LaTeX paper (single-column layout, 3.5 in wide, seaborn-v0_8-paper style).
"""

from __future__ import annotations

from pathlib import Path
from typing import Union

import matplotlib

matplotlib.use("Agg")
import matplotlib.patches as mpatches
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd

# ---------------------------------------------------------------------------
# Module-level constants
# ---------------------------------------------------------------------------

ENGINE_COLORS: dict[str, str] = {
    "naive": "#0072B2",
    "id": "#D55E00",
    "priority-id": "#009E73",
}

OUTCOME_COLORS: dict[str, str] = {
    "solved": "#4CAF50",
    "timeout": "#F44336",
    "max_steps": "#FF9800",
    "max_biconditionals": "#9C27B0",
    "max_depth": "#2196F3",
    "quantifier_budget": "#795548",
    "other": "#9E9E9E",
}

OUTCOME_ORDER: list[str] = [
    "solved",
    "timeout",
    "max_steps",
    "max_biconditionals",
    "max_depth",
    "quantifier_budget",
    "other",
]

HATCH_PATTERNS: dict[str, str] = {
    "naive": "",
    "id": "///",
    "priority-id": "xxx",
}

_STYLE = "seaborn-v0_8-paper"
_FONT_SIZE = 10
_FIG_WIDTH = 3.5  # inches, single-column


# ---------------------------------------------------------------------------
# Plot 1: Outcome composition stacked bar chart
# ---------------------------------------------------------------------------


def plot_outcome_composition(
    engine_results: pd.DataFrame,
    output_path: Union[str, Path],
) -> None:
    """Stacked bar chart showing outcome composition per engine.

    One bar per engine on the x-axis.  The y-axis shows the absolute problem
    count.  Bars are stacked in ``OUTCOME_ORDER`` from bottom to top and
    hatched according to ``HATCH_PATTERNS``.  Outcomes with a zero total count
    across all engines are omitted.  The legend is placed outside the axes on
    the right.

    Parameters
    ----------
    engine_results:
        DataFrame with columns ``engine``, ``problem_id``, ``outcome``,
        ``elapsed_ms``, ``atoms``, ``formulae``.
    output_path:
        Destination path for the saved PDF.
    """
    output_path = Path(output_path)

    # Build a pivot: rows = engine, columns = outcome, values = count
    counts = (
        engine_results.groupby(["engine", "outcome"])
        .size()
        .unstack(fill_value=0)
    )

    # Keep only outcomes that exist in the data, in canonical order
    present_outcomes = [o for o in OUTCOME_ORDER if o in counts.columns]
    # Any outcome not in OUTCOME_ORDER lands in a virtual "other" bucket
    extra = [c for c in counts.columns if c not in OUTCOME_ORDER]
    if extra:
        counts["other"] = counts.get("other", 0) + counts[extra].sum(axis=1)
        counts = counts.drop(columns=extra)
        if "other" not in present_outcomes:
            present_outcomes.append("other")

    counts = counts.reindex(columns=present_outcomes, fill_value=0)

    # Remove outcomes whose column sums to zero
    non_zero = [o for o in present_outcomes if counts[o].sum() > 0]
    counts = counts[non_zero]

    engines = list(counts.index)

    with plt.style.context(_STYLE):
        fig, ax = plt.subplots(figsize=(_FIG_WIDTH, 3.0))
        plt.rcParams.update({"font.size": _FONT_SIZE})

        x = np.arange(len(engines))
        bottoms = np.zeros(len(engines))

        legend_handles: list[mpatches.Patch] = []
        for outcome in non_zero:
            values = counts[outcome].to_numpy(dtype=float)
            color = OUTCOME_COLORS.get(outcome, OUTCOME_COLORS["other"])
            bars = ax.bar(
                x,
                values,
                bottom=bottoms,
                color=color,
                label=outcome,
            )
            # Apply per-engine hatching on each bar segment
            for bar_patch, engine in zip(bars, engines):
                bar_patch.set_hatch(HATCH_PATTERNS.get(engine, ""))
                bar_patch.set_edgecolor("white")
            bottoms += values
            legend_handles.append(
                mpatches.Patch(facecolor=color, label=outcome)
            )

        ax.set_xticks(x)
        ax.set_xticklabels(engines)
        ax.set_xlabel("Engine")
        ax.set_ylabel("Problem count")
        ax.set_title("Outcome composition by engine")

        ax.legend(
            handles=legend_handles,
            bbox_to_anchor=(1.02, 1),
            loc="upper left",
            borderaxespad=0.0,
            frameon=False,
            fontsize=_FONT_SIZE - 1,
        )

        fig.tight_layout()
        fig.savefig(output_path, format="pdf", bbox_inches="tight")
        plt.close(fig)


# ---------------------------------------------------------------------------
# Plot 2: CDF of solve times
# ---------------------------------------------------------------------------


def plot_solve_time_cdf(
    engine_results: pd.DataFrame,
    timeout_ms: int,
    output_path: Union[str, Path],
) -> None:
    """CDF of solve times, one line per engine.

    The x-axis uses a log scale and is labeled in milliseconds.  The y-axis
    shows the fraction of *all* problems (not just solved ones) in each
    engine's run that are solved at or before that time.  Only rows with
    ``outcome == 'solved'`` contribute to the CDF; the denominator is the
    total number of problems for that engine.

    A vertical dotted grey line marks ``timeout_ms``.

    Parameters
    ----------
    engine_results:
        DataFrame with columns ``engine``, ``problem_id``, ``outcome``,
        ``elapsed_ms``, ``atoms``, ``formulae``.
    timeout_ms:
        Timeout threshold in milliseconds; drawn as a reference line.
    output_path:
        Destination path for the saved PDF.
    """
    output_path = Path(output_path)

    with plt.style.context(_STYLE):
        fig, ax = plt.subplots(figsize=(_FIG_WIDTH, 2.8))
        plt.rcParams.update({"font.size": _FONT_SIZE})

        for engine, group in engine_results.groupby("engine"):
            total = len(group)
            solved = group[group["outcome"] == "solved"]["elapsed_ms"].dropna()
            if solved.empty or total == 0:
                continue

            sorted_times = np.sort(solved.to_numpy())
            cdf_values = np.arange(1, len(sorted_times) + 1) / total

            color = ENGINE_COLORS.get(str(engine), "#333333")
            ax.step(
                sorted_times,
                cdf_values,
                where="post",
                color=color,
                label=str(engine),
                linewidth=1.5,
            )

        ax.axvline(
            timeout_ms,
            color="grey",
            linestyle=":",
            linewidth=1.0,
            label="timeout",
        )

        ax.set_xscale("log")
        ax.set_xlabel("Elapsed time (ms)")
        ax.set_ylabel("Fraction solved")
        ax.set_ylim(0.0, 1.0)
        ax.set_title("Solve-time CDF")

        ax.legend(frameon=False, fontsize=_FONT_SIZE - 1)

        fig.tight_layout()
        fig.savefig(output_path, format="pdf", bbox_inches="tight")
        plt.close(fig)


# ---------------------------------------------------------------------------
# Plot 3: Solve rate by atom-count bin
# ---------------------------------------------------------------------------


def plot_solve_rate_by_atom_bin(
    engine_results: pd.DataFrame,
    output_path: Union[str, Path],
) -> None:
    """Grouped bar chart of solve rate across quantile atom-count bins.

    Four quantile-based bins are computed from all non-null ``atoms`` values
    across all engines.  Each bin shows one bar per engine, coloured by
    ``ENGINE_COLORS`` and hatched by ``HATCH_PATTERNS``.  Problems with null
    ``atoms`` are excluded.

    Parameters
    ----------
    engine_results:
        DataFrame with columns ``engine``, ``problem_id``, ``outcome``,
        ``elapsed_ms``, ``atoms``, ``formulae``.
    output_path:
        Destination path for the saved PDF.
    """
    output_path = Path(output_path)

    df = engine_results.dropna(subset=["atoms"]).copy()
    df["atoms"] = pd.to_numeric(df["atoms"], errors="coerce")
    df = df.dropna(subset=["atoms"])

    # Build 4 quantile bins from all atom values (global, not per-engine)
    all_atoms = df["atoms"]
    bin_edges = np.unique(
        np.quantile(all_atoms, [0.0, 0.25, 0.5, 0.75, 1.0])
    )
    # Ensure we always have exactly 4 bins; fall back if too few unique edges
    if len(bin_edges) < 2:
        # Cannot bin — return silently
        return

    # Build human-readable labels
    labels: list[str] = []
    cut_edges: list[float] = list(bin_edges)
    for i in range(len(cut_edges) - 1):
        lo = int(np.floor(cut_edges[i]))
        hi = int(np.floor(cut_edges[i + 1]))
        labels.append(f"{lo}–{hi}")

    df["bin"] = pd.cut(
        df["atoms"],
        bins=cut_edges,
        labels=labels,
        include_lowest=True,
    )

    engines = sorted(df["engine"].unique())
    n_engines = len(engines)
    bin_categories = [str(lbl) for lbl in labels]

    # Compute solve rates per (engine, bin)
    solve_rates: dict[str, dict[str, float]] = {}
    for engine in engines:
        edf = df[df["engine"] == engine]
        rates: dict[str, float] = {}
        for b in bin_categories:
            sub = edf[edf["bin"].astype(str) == b]
            if len(sub) == 0:
                rates[b] = float("nan")
            else:
                rates[b] = (sub["outcome"] == "solved").sum() / len(sub)
        solve_rates[str(engine)] = rates

    with plt.style.context(_STYLE):
        fig, ax = plt.subplots(figsize=(_FIG_WIDTH, 2.8))
        plt.rcParams.update({"font.size": _FONT_SIZE})

        bar_width = 0.8 / max(n_engines, 1)
        x = np.arange(len(bin_categories))

        for i, engine in enumerate(engines):
            offsets = x + (i - n_engines / 2 + 0.5) * bar_width
            values = [solve_rates[str(engine)].get(b, float("nan")) for b in bin_categories]
            color = ENGINE_COLORS.get(str(engine), "#333333")
            ax.bar(
                offsets,
                values,
                width=bar_width * 0.9,
                color=color,
                hatch=HATCH_PATTERNS.get(str(engine), ""),
                edgecolor="white",
                label=str(engine),
            )

        ax.set_xticks(x)
        ax.set_xticklabels(bin_categories, fontsize=_FONT_SIZE - 1)
        ax.set_xlabel("Atom count bin")
        ax.set_ylabel("Solve rate")
        ax.set_ylim(0.0, 1.0)
        ax.set_title("Solve rate by atom count")

        ax.legend(frameon=False, fontsize=_FONT_SIZE - 1)

        fig.tight_layout()
        fig.savefig(output_path, format="pdf", bbox_inches="tight")
        plt.close(fig)


# ---------------------------------------------------------------------------
# Plot 4: Solve rate by TPTP difficulty bin
# ---------------------------------------------------------------------------


def plot_solve_rate_by_difficulty_bin(
    engine_results: pd.DataFrame,
    tptp_ratings: dict[str, float],
    output_path: Union[str, Path],
) -> None:
    """Grouped bar chart of solve rate across TPTP difficulty bins.

    ``engine_results`` is joined with ``tptp_ratings`` on ``problem_id``.
    Problems absent from ``tptp_ratings`` are excluded.  Bins are hard-coded
    to semantically meaningful TPTP difficulty levels:

    - ``"0.0"``: rating == 0.0
    - ``"0.01–0.25"``: 0.0 < rating <= 0.25
    - ``"0.26–0.5"``: 0.25 < rating <= 0.5
    - ``"0.51–0.75"``: 0.5 < rating <= 0.75
    - ``"0.76–1.0"``: 0.75 < rating <= 1.0

    Bins with zero problems across all engines are skipped.

    Parameters
    ----------
    engine_results:
        DataFrame with columns ``engine``, ``problem_id``, ``outcome``,
        ``elapsed_ms``, ``atoms``, ``formulae``.
    tptp_ratings:
        Mapping from ``problem_id`` to TPTP difficulty rating (0.0–1.0).
    output_path:
        Destination path for the saved PDF.
    """
    output_path = Path(output_path)

    # Join ratings
    ratings_series = pd.Series(tptp_ratings, name="rating")
    df = engine_results.join(ratings_series, on="problem_id", how="inner")
    if df.empty:
        return

    # Hard-coded bin definitions: (label, predicate function)
    bin_defs: list[tuple[str, object]] = [
        ("0.0",        lambda r: r == 0.0),
        ("0.01–0.25", lambda r: (r > 0.0) & (r <= 0.25)),
        ("0.26–0.5",  lambda r: (r > 0.25) & (r <= 0.5)),
        ("0.51–0.75", lambda r: (r > 0.5) & (r <= 0.75)),
        ("0.76–1.0",  lambda r: (r > 0.75) & (r <= 1.0)),
    ]

    def assign_bin(rating_col: pd.Series) -> pd.Series:
        result = pd.Series(index=rating_col.index, dtype=object)
        for label, pred in bin_defs:
            mask = pred(rating_col)
            result[mask] = label
        return result

    df = df.copy()
    df["diff_bin"] = assign_bin(df["rating"])

    # Remove bins that have zero problems
    all_labels = [label for label, _ in bin_defs]
    non_empty_bins = [b for b in all_labels if (df["diff_bin"] == b).sum() > 0]
    df = df[df["diff_bin"].isin(non_empty_bins)]

    engines = sorted(df["engine"].unique())
    n_engines = len(engines)

    # Compute solve rates
    solve_rates: dict[str, dict[str, float]] = {}
    for engine in engines:
        edf = df[df["engine"] == engine]
        rates: dict[str, float] = {}
        for b in non_empty_bins:
            sub = edf[edf["diff_bin"] == b]
            if len(sub) == 0:
                rates[b] = float("nan")
            else:
                rates[b] = (sub["outcome"] == "solved").sum() / len(sub)
        solve_rates[str(engine)] = rates

    with plt.style.context(_STYLE):
        fig, ax = plt.subplots(figsize=(_FIG_WIDTH, 2.8))
        plt.rcParams.update({"font.size": _FONT_SIZE})

        bar_width = 0.8 / max(n_engines, 1)
        x = np.arange(len(non_empty_bins))

        for i, engine in enumerate(engines):
            offsets = x + (i - n_engines / 2 + 0.5) * bar_width
            values = [solve_rates[str(engine)].get(b, float("nan")) for b in non_empty_bins]
            color = ENGINE_COLORS.get(str(engine), "#333333")
            ax.bar(
                offsets,
                values,
                width=bar_width * 0.9,
                color=color,
                hatch=HATCH_PATTERNS.get(str(engine), ""),
                edgecolor="white",
                label=str(engine),
            )

        ax.set_xticks(x)
        ax.set_xticklabels(non_empty_bins, fontsize=_FONT_SIZE - 2, rotation=15, ha="right")
        ax.set_xlabel("TPTP difficulty rating")
        ax.set_ylabel("Solve rate")
        ax.set_ylim(0.0, 1.0)
        ax.set_title("Solve rate by difficulty")

        ax.legend(frameon=False, fontsize=_FONT_SIZE - 1)

        fig.tight_layout()
        fig.savefig(output_path, format="pdf", bbox_inches="tight")
        plt.close(fig)
