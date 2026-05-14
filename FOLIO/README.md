# FOLIO pipeline

Exports each [FOLIO](https://huggingface.co/datasets/yale-nlp/FOLIO) example's
premises and conclusion (human-authored FOL) as two TPTP problems, runs
the project's prover over them, and produces a prediction TSV plus a
confusion-matrix summary. Dataset rationale and result interpretation:
`report/sections/dataset.tex` (*FOLIO*) and
`report/sections/experimental_setup.tex`.

## How predictions are derived

For each example with gold label `true` / `false` / `uncertain`:

- `{example}__entails.p` asks whether `premises ⊢ conclusion`.
- `{example}__refutes.p` asks whether `premises ⊢ ¬conclusion`.

`folio/evaluate.py:predict` combines the two prover statuses into one label:

| `entails` provable | `refutes` provable | Predicted      |
| ------------------ | ------------------ | -------------- |
| yes                | no                 | `true`         |
| no                 | yes                | `false`        |
| no                 | no                 | `unknown`      |
| yes                | yes                | `inconsistent` |
| timeout / error on either side          | `undetermined` |

## Install

FOLIO on HuggingFace is gated: request access on the dataset page, then
`huggingface-cli login`. After that, from the repo root:

```powershell
pip install -e .\FOLIO[dev]
```

This installs the `folio` console script (`pyproject.toml`) plus `pytest`.

## Running

```powershell
# Validation split, priority-id engine, 1000ms timeout (defaults).
folio

# Override any flag. `python -m folio ...` is equivalent.
folio --split validation --engine priority-id --timeout-ms 1000 --limit 5

# Skip the prover (just regenerate the TPTP exports under FOLIO/generated/).
folio --export-only
```

`--engine` is forwarded to the Rust prover; the report evaluates `naive`,
`id`, and `priority-id`.

## Pipeline

1. **Load** the requested split(s) from `yale-nlp/FOLIO` (`folio/dataset.py`).
2. **Translate** each example's FOL strings into TPTP `fof(...)` formulae
   (`folio/formula.py`). Supported examples land in `FOLIO/generated/`,
   listed in `metadata.jsonl`; skipped examples go in `unsupported.jsonl`
   with the reason.
3. **Invoke the prover** on `FOLIO/generated/` (`folio/prover.py`):
   ```text
   cargo run --quiet --manifest-path ..\theorem_prover\Cargo.toml -- prove \
       --problem-class mixed --format tsv \
       --timeout-ms <ms> --engine <engine> \
       --persist FOLIO\folio-results.db \
       --run-label folio-<utc-timestamp> \
       FOLIO\generated
   ```
   Run `prove --help` from the prover crate for the full flag list.
4. **Evaluate** (`folio/evaluate.py`): join the new run from
   `folio-results.db` against the exported entails/refutes pairs and write
   `folio-predictions.tsv` (one row per example) and `folio-summary.tsv`
   (confusion-matrix counts).

## Reproducing the report numbers

The committed TSVs come from running `folio` once per engine after a
fresh dataset checkout. Each run appends a row to `folio-results.db` with
its own `--run-label`; the TSVs reflect only the latest run, so copy them
aside between invocations:

```powershell
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
