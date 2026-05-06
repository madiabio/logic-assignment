from __future__ import annotations

import math
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
import pandas as pd

from analysis_core import OUTCOME_ORDER


def save_bar_chart(df: pd.DataFrame, path: Path, title: str, ylabel: str, stacked: bool = False) -> None:
    fig, ax = plt.subplots(figsize=(10, 6))
    if stacked:
        bottom = None
        colors = {
            "solved": "#2a9d8f",
            "timeout": "#e76f51",
            "unknown": "#f4a261",
            "other": "#6c757d",
            "no_data": "#264653",
            "max_depth": "#457b9d",
            "max_steps": "#1d3557",
            "biconditional_cap": "#e9c46a",
            "quantifier_budget": "#8d99ae",
        }
        stacked_columns = [column for column in OUTCOME_ORDER if column in df.columns]
        if not stacked_columns:
            stacked_columns = list(df.columns)
        for column in stacked_columns:
            values = df[column].to_numpy()
            ax.bar(df.index, values, bottom=bottom, label=column, color=colors.get(column))
            bottom = values if bottom is None else bottom + values
        ax.legend(frameon=False, ncol=3)
    else:
        bars = ax.bar(df.index, df.iloc[:, 0].to_numpy(), color="#2a9d8f")
        ax.bar_label(bars, fmt="%.0f%%", padding=3)

    ax.set_title(title)
    ax.set_ylabel(ylabel)
    ax.set_xlabel("")
    ax.spines["top"].set_visible(False)
    ax.spines["right"].set_visible(False)
    fig.tight_layout()
    fig.savefig(path, dpi=160)
    plt.close(fig)


def save_boxplot(df: pd.DataFrame, path: Path, title: str, ylabel: str) -> None:
    fig, ax = plt.subplots(figsize=(10, 6))
    data = [df[col].dropna().to_numpy() for col in df.columns]
    ax.boxplot(data, labels=df.columns, showfliers=False)
    ax.set_title(title)
    ax.set_ylabel(ylabel)
    ax.set_xlabel("")
    ax.set_yscale("log")
    ax.spines["top"].set_visible(False)
    ax.spines["right"].set_visible(False)
    fig.tight_layout()
    fig.savefig(path, dpi=160)
    plt.close(fig)


def save_heatmap(matrix: pd.DataFrame, path: Path, title: str, annotation: pd.DataFrame | None = None) -> None:
    fig, ax = plt.subplots(figsize=(8, 6))
    data = matrix.to_numpy(dtype=float)
    im = ax.imshow(data, vmin=0, vmax=1, cmap="viridis")
    ax.set_xticks(range(len(matrix.columns)))
    ax.set_yticks(range(len(matrix.index)))
    ax.set_xticklabels(matrix.columns)
    ax.set_yticklabels(matrix.index)
    ax.set_title(title)
    for i in range(matrix.shape[0]):
        for j in range(matrix.shape[1]):
            value = data[i, j]
            if math.isnan(value):
                label = "n/a"
            elif annotation is not None:
                label = str(annotation.iloc[i, j])
            else:
                label = f"{value:.2f}"
            ax.text(j, i, label, ha="center", va="center", color="white" if not math.isnan(value) and value > 0.5 else "black")
    fig.colorbar(im, ax=ax, label="Win share")
    fig.tight_layout()
    fig.savefig(path, dpi=160)
    plt.close(fig)


def save_histogram(values: pd.Series, path: Path, title: str, xlabel: str) -> None:
    fig, ax = plt.subplots(figsize=(10, 6))
    ax.hist(values.dropna().to_numpy(), bins=30, color="#457b9d", edgecolor="white")
    ax.axvline(0, color="#111111", linewidth=1, linestyle="--")
    ax.set_title(title)
    ax.set_xlabel(xlabel)
    ax.set_ylabel("Problems")
    ax.spines["top"].set_visible(False)
    ax.spines["right"].set_visible(False)
    fig.tight_layout()
    fig.savefig(path, dpi=160)
    plt.close(fig)
