from __future__ import annotations

import json
import pathlib
import re
from dataclasses import dataclass
from typing import Iterable

from folio.formula import UnsupportedFormula, convert_fol_formula


@dataclass
class ExportedExample:
    split: str
    story_id: str
    example_id: str
    label: str
    entails_file: pathlib.Path
    refutes_file: pathlib.Path


def safe_example_id(row: dict, index: int | None = None) -> str:
    raw = row.get("example-id") or row.get("example_id") or row.get("id") or f"row_{index}"
    safe = re.sub(r"[^A-Za-z0-9_]+", "_", str(raw)).strip("_")
    return safe or f"row_{index}"


def row_label(row: dict) -> str:
    return str(row.get("label", "")).strip().lower()


def folio_premises(row: dict) -> list[str]:
    premises = row.get("premises-FOL") or row.get("premises_fol") or []
    if isinstance(premises, str):
        try:
            parsed = json.loads(premises)
            if isinstance(parsed, list):
                return [str(item) for item in parsed]
        except json.JSONDecodeError:
            return [line.strip() for line in premises.splitlines() if line.strip()]
    return [str(item) for item in premises]


def folio_conclusion(row: dict) -> str:
    return str(row.get("conclusion-FOL") or row.get("conclusion_fol") or "")


def write_jsonl(path: pathlib.Path, records: Iterable[dict]) -> None:
    with path.open("w", encoding="utf-8") as handle:
        for record in records:
            handle.write(json.dumps(record, ensure_ascii=False) + "\n")


def export_rows(rows: Iterable[dict], split: str, out_dir: pathlib.Path) -> list[ExportedExample]:
    out_dir.mkdir(parents=True, exist_ok=True)
    metadata: list[dict] = []
    unsupported: list[dict] = []
    exported: list[ExportedExample] = []

    for index, row in enumerate(rows):
        example_id = str(row.get("example-id") or row.get("example_id") or row.get("id") or f"row_{index}")
        story_id = str(row.get("story-id") or row.get("story_id") or "")
        safe_id = safe_example_id(row, index)
        try:
            premises = []
            for premise_index, premise in enumerate(folio_premises(row), start=1):
                try:
                    premises.append((premise_index, convert_fol_formula(premise)))
                except UnsupportedFormula as err:
                    unsupported.append(
                        {
                            "split": split,
                            "example_id": example_id,
                            "field": f"premises-FOL[{premise_index - 1}]",
                            "formula": premise,
                            "reason": str(err),
                        }
                    )
                    raise
            raw_conclusion = folio_conclusion(row)
            try:
                conclusion = convert_fol_formula(raw_conclusion)
            except UnsupportedFormula as err:
                unsupported.append(
                    {
                        "split": split,
                        "example_id": example_id,
                        "field": "conclusion-FOL",
                        "formula": raw_conclusion,
                        "reason": str(err),
                    }
                )
                raise
        except UnsupportedFormula:
            continue

        entails_file = out_dir / f"{split}__{safe_id}__entails.p"
        refutes_file = out_dir / f"{split}__{safe_id}__refutes.p"
        entails_lines = [f"fof(premise_{i},axiom,{formula})." for i, formula in premises]
        entails_lines.append(f"fof(conclusion,conjecture,{conclusion}).")
        refutes_lines = [f"fof(premise_{i},axiom,{formula})." for i, formula in premises]
        refutes_lines.append(f"fof(conclusion_negated,conjecture,~({conclusion})).")
        entails_file.write_text("\n".join(entails_lines) + "\n", encoding="utf-8")
        refutes_file.write_text("\n".join(refutes_lines) + "\n", encoding="utf-8")

        item = ExportedExample(
            split=split,
            story_id=story_id,
            example_id=example_id,
            label=row_label(row),
            entails_file=entails_file,
            refutes_file=refutes_file,
        )
        exported.append(item)
        metadata.append(
            {
                "split": split,
                "story_id": story_id,
                "example_id": example_id,
                "label": item.label,
                "entails_file": str(entails_file),
                "refutes_file": str(refutes_file),
            }
        )

    write_jsonl(out_dir / "metadata.jsonl", metadata)
    write_jsonl(out_dir / "unsupported.jsonl", unsupported)
    return exported
