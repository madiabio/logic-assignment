# FOLIO pipeline

Downloads the [FOLIO](https://huggingface.co/datasets/yale-nlp/FOLIO) dataset,
converts each example's premises and conclusion (human-authored FOL) into
two TPTP problems, runs the project's theorem prover over them, and writes
a per-example prediction TSV plus a summary confusion-matrix TSV.

For the dataset rationale and the role this pipeline plays in the report's
evaluation, see `report/sections/dataset.tex` (*FOLIO*) and
`report/sections/experimental_setup.tex`.

## How predictions are derived

For each FOLIO example with gold label `true` / `false` / `uncertain`:

- `{example}__entails.p` asks the prover whether `premises ⊢ conclusion`.
- `{example}__refutes.p` asks whether `premises ⊢ ¬conclusion`.

The label is derived from the pair of prover statuses
(`folio/evaluate.py:predict`):

| `entails` provable | `refutes` provable | Predicted      |
| ------------------ | ------------------ | -------------- |
| yes                | no                 | `true`         |
| no                 | yes                | `false`        |
| no                 | no                 | `unknown`      |
| yes                | yes                | `inconsistent` |
| timeout / error on either side          | `undetermined` |

## Prerequisites

1. **Python 3.12+.** Pinned by `pyproject.toml` (`requires-python = ">=3.12"`).
2. **Rust toolchain (`cargo`).** The prover is invoked as a subprocess via
   `cargo run --manifest-path ../theorem_prover/Cargo.toml -- prove ...`
   (`folio/prover.py`), so `cargo` must be on `PATH` and the prover must
   build from the repo root.
3. **Hugging Face account with access to `yale-nlp/FOLIO`.** The dataset
   is gated. Request access on the dataset page, then authenticate:
   ```powershell
   pip install huggingface_hub
   huggingface-cli login
   ```

## Install

From the repository root:

```powershell
pip install -e .\FOLIO[dev]
```

This installs the `folio` console script (declared in
`FOLIO/pyproject.toml`) and `pytest`.

## Running

```powershell
# Defaults: validation split, priority-id engine, 1000ms timeout,
# writes TPTP problems to FOLIO\generated\, results to FOLIO\folio-results.db,
# TSVs to FOLIO\folio-predictions.tsv and FOLIO\folio-summary.tsv.
folio

# Explicit form (equivalent to the above).
folio `
    --split        validation `
    --out-dir      FOLIO\generated `
    --db           FOLIO\folio-results.db `
    --engine       priority-id `
    --timeout-ms   1000

# Export TPTP problems without invoking the prover (useful for debugging
# the FOL-to-TPTP translation in folio/formula.py).
folio --export-only

# Smoke run on the first 5 supported examples.
folio --limit 5
```

`python -m folio ...` accepts the same flags as `folio ...`.

### Engines

`--engine` is forwarded to the Rust prover. The report evaluates `naive`,
`id`, and `priority-id`; see the root README for what each means.

### What each step does

1. **Load** the requested split(s) from `yale-nlp/FOLIO` (`folio/dataset.py`).
2. **Translate** each example's `premises-FOL` and `conclusion-FOL` strings
   into TPTP `fof(...)` formulae (`folio/formula.py`). Examples using FOL
   features the translator does not yet support are skipped and listed in
   `generated/unsupported.jsonl`; supported examples are recorded in
   `generated/metadata.jsonl` (`folio/export.py`).
3. **Invoke the prover** on `FOLIO/generated/` (`folio/prover.py`):
   ```text
   cargo run --quiet --manifest-path ..\theorem_prover\Cargo.toml -- prove \
       --problem-class mixed \
       --format        tsv \
       --timeout-ms    <timeout-ms> \
       --engine        <engine> \
       --persist       FOLIO\folio-results.db \
       --run-label     folio-<utc-timestamp> \
       FOLIO\generated
   ```
   Run `cargo run --manifest-path ..\theorem_prover\Cargo.toml -- prove --help`
   for the full prover flag list.
4. **Evaluate** (`folio/evaluate.py`). Joins the new run from the results DB
   against the exported `entails` / `refutes` pairs and writes the two TSVs.

### Outputs

`FOLIO/generated/` is **gitignored**; the TSVs and the SQLite DB are
committed so the report's numbers stay reproducible without rerunning the
prover.

| File                                | State      | Contents                                                 |
| ----------------------------------- | ---------- | -------------------------------------------------------- |
| `FOLIO/generated/*.p`               | gitignored | TPTP problems, two per supported example.                |
| `FOLIO/generated/metadata.jsonl`    | gitignored | One JSON object per supported example.                   |
| `FOLIO/generated/unsupported.jsonl` | gitignored | Examples skipped due to unsupported FOL features.        |
| `FOLIO/folio-results.db`            | committed  | SQLite log of every prover run (engines, timings, status). |
| `FOLIO/folio-predictions.tsv`       | committed  | Per-example predictions for the latest run.              |
| `FOLIO/folio-summary.tsv`           | committed  | `(prediction, gold)` confusion-matrix counts, latest run. |

## Reproducing the report numbers

The TSVs cited in `report/sections/experimental_setup.tex` were produced by
running `folio` once per engine after a fresh dataset checkout. To
regenerate from scratch:

```powershell
# 1. (Optional) clear the gitignored exports and the committed outputs.
Remove-Item -Recurse -Force FOLIO\generated `
    -ErrorAction SilentlyContinue
Remove-Item FOLIO\folio-results.db, `
            FOLIO\folio-predictions.tsv, `
            FOLIO\folio-summary.tsv `
    -ErrorAction SilentlyContinue

# 2. Run the pipeline once per engine. Each run appends a row to the DB
#    with its own --run-label; the TSVs reflect only the most recent run,
#    so copy them aside between invocations.
foreach ($engine in @('naive', 'id', 'priority-id')) {
    folio --engine $engine
    Copy-Item FOLIO\folio-predictions.tsv "FOLIO\folio-predictions.$engine.tsv"
    Copy-Item FOLIO\folio-summary.tsv     "FOLIO\folio-summary.$engine.tsv"
}
```

## Tests

```powershell
pytest FOLIO\tests
```

Covers the FOL-to-TPTP translator (`test_formula.py`), the export layer
(`test_export.py`), the evaluation logic (`test_evaluate.py`), and the
SQLite reader (`test_db.py`).
