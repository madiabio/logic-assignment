from __future__ import annotations

import csv
import pathlib

from folio.db import ResultDetail, RunConstraints, detail_for
from folio.export import ExportedExample

SUMMARY_TSV = pathlib.Path("FOLIO/folio-summary.tsv")
PREDICTIONS_TSV = pathlib.Path("FOLIO/folio-predictions.tsv")


def is_provable(status: str) -> bool:
    return status == "provable"


def predict(entails_status: str, refutes_status: str) -> str:
    if entails_status in {"timeout", "error", "cancelled", "not_implemented", "missing"}:
        return "undetermined"
    if refutes_status in {"timeout", "error", "cancelled", "not_implemented", "missing"}:
        return "undetermined"
    entails = is_provable(entails_status)
    refutes = is_provable(refutes_status)
    if entails and not refutes:
        return "true"
    if refutes and not entails:
        return "false"
    if not entails and not refutes:
        return "unknown"
    return "inconsistent"


def _fmt_detail(d: ResultDetail) -> str:
    parts = [d.status]
    if d.elapsed_ms > 0:
        parts.append(f"({d.elapsed_ms}ms)")
    if d.unknown_reason:
        parts.append(f"[{d.unknown_reason}]")
    return " ".join(parts)


def write_evaluation(
    exported: list[ExportedExample],
    details: dict[str, ResultDetail],
    constraints: RunConstraints | None = None,
) -> None:
    SUMMARY_TSV.parent.mkdir(parents=True, exist_ok=True)
    counts: dict[tuple[str, str], int] = {}
    prediction_rows = []
    eval_items = []

    for item in exported:
        entails_detail = detail_for(details, item.entails_file)
        refutes_detail = detail_for(details, item.refutes_file)
        predicted = predict(entails_detail.status, refutes_detail.status)
        gold = item.label
        counts[(predicted, gold)] = counts.get((predicted, gold), 0) + 1
        prediction_rows.append(
            {
                "split": item.split,
                "story_id": item.story_id,
                "example_id": item.example_id,
                "gold": gold,
                "prediction": predicted,
                "entails_status": entails_detail.status,
                "refutes_status": refutes_detail.status,
                "entails_file": str(item.entails_file),
                "refutes_file": str(item.refutes_file),
            }
        )
        eval_items.append((item, entails_detail, refutes_detail, predicted, gold))

    # --- printed output ---
    if constraints:
        print(f"\n=== FOLIO Evaluation: {constraints.label} ===")
        print(
            f"  engine={constraints.engine}"
            f"  timeout={constraints.timeout_ms}ms"
            f"  max_depth={constraints.max_depth}"
            f"  max_steps={constraints.max_steps}"
            f"  max_fresh_terms_per_quantifier={constraints.max_fresh_terms_per_quantifier}"
            f"  problem_class={constraints.problem_class}"
        )
    else:
        print("\n=== FOLIO Evaluation ===")
    print()

    col_id, col_gold, col_pred, col_entails = 15, 11, 14, 28
    header = (
        f"{'example_id':<{col_id}}  {'gold':<{col_gold}}  {'prediction':<{col_pred}}"
        f"  {'entails':<{col_entails}}  refutes"
    )
    print(header)
    print("-" * (len(header) + 10))
    for item, entails_detail, refutes_detail, predicted, gold in eval_items:
        print(
            f"{item.example_id:<{col_id}}  {gold:<{col_gold}}  {predicted:<{col_pred}}"
            f"  {_fmt_detail(entails_detail):<{col_entails}}  {_fmt_detail(refutes_detail)}"
        )

    print()
    print("=== Summary ===")
    for (prediction_value, gold), count in sorted(counts.items()):
        print(f"  {prediction_value:<14}  {gold:<10}  {count}")
    print()

    # --- TSV output ---
    with PREDICTIONS_TSV.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(
            handle,
            fieldnames=list(prediction_rows[0].keys()) if prediction_rows else [
                "split", "story_id", "example_id", "gold", "prediction",
                "entails_status", "refutes_status", "entails_file", "refutes_file",
            ],
            delimiter="\t",
        )
        writer.writeheader()
        writer.writerows(prediction_rows)

    with SUMMARY_TSV.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.writer(handle, delimiter="\t")
        writer.writerow(["prediction", "gold", "count"])
        for (prediction_value, gold), count in sorted(counts.items()):
            writer.writerow([prediction_value, gold, count])
