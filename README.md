# Automated First-Order Logic Prover

A Rust implementation of Hou's LK′ backward sequent search for first-order
logic, with two extensions that address its principal failure modes: a
6-class **priority schedule** that orders rule applications by productivity,
and an **iterative-deepening** envelope around the depth-first kernel. All
three engines (`naive`, `id`, `priority-id`) share the same parser, sequent
representation, and `backwards_search` kernel; only the scheduler is
swapped. The prover is evaluated on three datasets (a TPTP subset, FOLIO,
and a synthetic FOF benchmark) totalling 1{,}370 problems.

The full motivation, calculus, engine descriptions, dataset construction,
and results live in the report. This README is a navigation map and a
reproducibility recipe; it does not duplicate the report's prose.

| Report section (`report/sections/`)   | What it covers                            |
| ------------------------------------- | ----------------------------------------- |
| `introduction.tex`                    | LK′ background, the two extensions.     |
| `background.tex`                      | Sequent calculus, related provers.        |
| `proposed-approach.tex`               | Priority schedule and iterative deepening. |
| `implementation.tex`                  | Rust crates, run configuration, hardware. |
| `dataset.tex`                         | TPTP, FOLIO, and synthetic datasets.      |
| `experimental_setup.tex`              | Results tables and per-dataset analysis.  |
| `discussion-and-conclusion.tex`       | Findings and limitations.                 |

## Project layout

| Path                            | Contents                                                       |
| ------------------------------- | -------------------------------------------------------------- |
| `theorem_prover/`               | The Rust prover (Cargo project). Subcommands: `prove`, `rules`. |
| `subset_descriptions/`          | TPTP subset listings; line 1 of each file records its `tptp2t` command. |
| `scripts/`                      | Subset formatting, engine sweep, synthetic generator. See `scripts/README.md`. |
| `generated-tests/`              | Committed copy of the synthetic FOF benchmark.                 |
| `tests/`                        | Pytest tests for the synthetic generator.                      |
| `FOLIO/`                        | FOLIO natural-language-to-FOL pipeline. See `FOLIO/README.md`. |
| `analysis/`                     | SQLite result DBs and the analysis notebook backing the report's figures. |
| `report/`                       | LaTeX source.                                                  |

## Building and running the prover

All `cargo` commands must be run **from inside `theorem_prover/`**, because
the committed `config.toml` resolves `tptp_root` and `default_subset_file`
relative to that directory:

```powershell
cd theorem_prover
copy config.toml.example config.toml   # then edit paths and limits
cargo build --release
cargo run -- prove --help              # authoritative flag reference
cargo run -- rules --help
```

A standalone run looks like:

```powershell
cd theorem_prover
cargo run --release -- prove `
    --subset-file   ..\subset_descriptions\medium_problems.txt `
    --engine        priority-id `
    --problem-class provable `
    --format        tsv
```

`--subset-file` and `--tptp-root` default to the values in `config.toml`,
so a fully configured run is just `cargo run --release -- prove --engine
priority-id --problem-class provable`. Results land in the SQLite file
named by `results_db` in `config.toml` (or `--persist <path>`), one row
per `(run, problem)`.

The three engines are `naive` (Hou's baseline DFS), `id` (iterative
deepening over the same DFS), and `priority-id` (iterative deepening with
the 6-class priority schedule).

## Reproducing the report's artefacts

### TPTP subsets (`subset_descriptions/`)

Each subset file's **first line** records the exact `tptp2T` / `tptp2t`
selection command that produced it, run inside a TPTP-v9 distribution:

- `easy_problems.txt` — FOF/THM, rating 0, ≤ 50 formulae, ≤ 150 atoms, no equality or arithmetic.
- `medium_problems.txt` — FOF/THM, unbounded rating, same complexity bounds.
- `medium_problems_countersatisfiable.txt` — FOF/CSA, unbounded rating, same bounds.

To rebuild a subset, re-run the command on line 1 inside a TPTP release,
then sort/format the output with `scripts/Format_TPTP_Subset.ps1`. See
`scripts/README.md` for the invocation.

### Synthetic benchmarks (`generated-tests/`)

```powershell
python scripts/generate_fof_benchmarks.py
python tests/test_fof_generation.py
```

Construction rationale: `report/sections/dataset.tex` (*Synthetic*).
Family-by-family description and tier sizes: `scripts/README.md`.

### Engine comparison runs

`scripts/run_all_engines.ps1` invokes `prove` once per engine (`naive`,
`id`, `priority-id`) and forwards any extra flags. It changes into
`theorem_prover/` automatically, so it can be called from the repo root.
This produced each per-engine results table in the report.

```powershell
.\scripts\run_all_engines.ps1 `
    --subset-file   ..\subset_descriptions\medium_problems.txt `
    --problem-class provable
```

### FOLIO

See `FOLIO/README.md` for the FOLIO download, FOL-to-TPTP export, prover
invocation, and prediction TSV pipeline.

## Analysis

`analysis/` holds the SQLite result DBs produced by each evaluation run
(`folio-results.db`, `generated-results.db`, `tptp-provable-results.db`,
`tptp-unprovable-results.db`) plus the notebook (`notebook.ipynb`) that
joins them into the CSVs and figures cited in the report.
