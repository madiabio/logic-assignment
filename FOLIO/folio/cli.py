"""Download FOLIO, export supported FOL annotations to TPTP, and evaluate them."""
from __future__ import annotations

import argparse
import pathlib
import sys

from folio.dataset import load_folio_dataset
from folio.db import fetch_run_constraints, latest_run_id, result_details
from folio.evaluate import write_evaluation
from folio.export import ExportedExample, export_rows
from folio.prover import run_prover

DEFAULT_OUT_DIR = pathlib.Path("FOLIO/generated")
DEFAULT_DB = pathlib.Path("FOLIO/folio-results.db")


def parse_args(argv: list[str] | None = None) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--split", choices=["train", "validation", "all"], default="validation")
    parser.add_argument("--out-dir", type=pathlib.Path, default=DEFAULT_OUT_DIR)
    parser.add_argument("--db", type=pathlib.Path, default=DEFAULT_DB)
    parser.add_argument("--timeout-ms", type=int, default=1000)
    parser.add_argument("--engine", default="priority-id")
    parser.add_argument("--limit", type=int)
    parser.add_argument("--export-only", action="store_true")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = parse_args(argv)
    exported: list[ExportedExample] = []
    for split, rows in load_folio_dataset(args.split):
        selected_rows = list(rows)
        if args.limit is not None:
            selected_rows = selected_rows[: args.limit]
        exported.extend(export_rows(selected_rows, split, args.out_dir))

    if args.export_only:
        write_evaluation(exported, {})
        print(f"Exported {len(exported)} supported FOLIO example(s) to {args.out_dir}")
        return 0

    before_run_id = latest_run_id(args.db)
    result = run_prover(args.out_dir, args.db, args.timeout_ms, args.engine)
    if result.stdout:
        print(result.stdout, end="")
    if result.stderr:
        print(result.stderr, end="", file=sys.stderr)
    after_run_id = latest_run_id(args.db)
    run_id = after_run_id if after_run_id != before_run_id else after_run_id
    details = result_details(args.db, run_id)
    constraints = fetch_run_constraints(args.db, run_id)
    write_evaluation(exported, details, constraints)
    return result.returncode
