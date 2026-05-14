# Logic Assignment

A Rust implementation of Hou's LK$'$ backward sequent search, evaluated on
TPTP, FOLIO, and a synthetic FOF benchmark. The motivation, calculus,
engine descriptions, dataset rationale, and experimental setup live in the
report (`report/sections/*.tex`):

- `introduction.tex` — what LK$'$ is and what this project changes.
- `proposed-approach.tex` — priority schedule and iterative deepening.
- `dataset.tex` — TPTP, FOLIO, and synthetic dataset construction.
- `implementation.tex` — Rust crates used, run configuration, hardware.
- `experimental_setup.tex` — numbers cited in the report.

This README only documents how to reproduce the artefacts in the repo.

## Repository layout

Paths marked **(gitignored)** are not committed; you have to obtain or
regenerate them locally (see `.gitignore` for the full list).

| Path                       | Contents                                                          |
| -------------------------- | ----------------------------------------------------------------- |
| `theorem_prover/`          | Rust prover (Cargo project). Subcommands: `prove`, `rules`.       |
| `theorem_prover/config.toml.example` | Template for `config.toml` (the real file is gitignored). |
| `subset_descriptions/`     | TPTP subset listings; line 1 of each file records its `tptp2t` command. |
| `scripts/`                 | Subset formatting, engine sweep, FOF benchmark generator.         |
| `generated-tests/`         | AI-generated FOF benchmark problems (regenerable from `scripts/`). |
| `tests/`                   | Pytest tests for the FOF benchmark generator.                     |
| `FOLIO/`                   | FOLIO pipeline (see `FOLIO/README.md`).                           |
| `report/`                  | LaTeX source.                                                     |
| `TPTP-v9.2.1/`             | **(gitignored)** Local copy of the TPTP-v9 release.               |
| `tptp_problems/`, `tptp_problem*/` | **(gitignored)** Staged `.p` files for ad-hoc runs.       |
| `theorem_prover/config.toml` | **(gitignored)** Local prover config; copy from `.example`.     |
| `results.db`               | **(gitignored)** SQLite run log written by the prover.            |
| `FOLIO/generated/`         | **(gitignored)** TPTP problems FOLIO exports per example.         |
| `misc/`, `docs/`           | **(gitignored)** Scratch directories.                             |

## Prerequisites

1. **Rust toolchain** (`cargo`). Build with `cargo build --release --manifest-path theorem_prover/Cargo.toml`.
2. **A local TPTP release** under `TPTP-v9.2.1/` (or pass `--tptp-root <path>`).
   Download from <https://www.tptp.org/>; the repo expects v9.2.1.
3. **Python 3.12+** for the synthetic-problem generator and the FOLIO pipeline.
4. **`theorem_prover/config.toml`**. Copy `config.toml.example`, then fill in
   the paths and limits. The committed example mirrors the values cited in
   the report (`timeout_ms = 1000`, `max_depth = 128`,
   `max_steps = 10000`, `max_biconditionals = 6`,
   `max_fresh_terms_per_quantifier = 1`).

## TPTP subsets

The files in `subset_descriptions/` were produced by the TPTP distribution's
`tptp2T` / `tptp2t` selection tool, run against a TPTP v9 release. The exact
command used for each subset is recorded as the **first line** of the file:

| File                                                       | Selection (line 1)                                                       |
| ---------------------------------------------------------- | ------------------------------------------------------------------------ |
| `subset_descriptions/easy_problems.txt`                    | FOF/THM, rating 0, $\le 50$ formulae, $\le 150$ atoms, no equality or arithmetic. |
| `subset_descriptions/medium_problems.txt`                  | FOF/THM, unbounded rating, same complexity bounds.                       |
| `subset_descriptions/medium_problems_countersatisfiable.txt` | FOF/CSA, unbounded rating, same complexity bounds.                     |

To regenerate, run the command on line 1 of the file inside any TPTP
distribution that ships `tptp2T` / `tptp2t`, then pipe its output through
`scripts/Format_TPTP_Subset.ps1` to sort by ascending atom count and emit
the canonical header:

```powershell
.\scripts\Format_TPTP_Subset.ps1 `
    -InputFile  .\tptp_medium_countersat.txt `
    -Difficulty medium `
    -CompareFile .\subset_descriptions\medium_problems.txt
```

`-CompareFile` reports problems that overlap the comparison subset, which is
how the THM and CSA medium subsets were checked for disjointness.

## Synthetic FOF benchmarks

`generated-tests/*.p` was produced by `scripts/generate_fof_benchmarks.py`.
The construction (four tiers: easy 32, medium 60, hard 60, expert 50,
across implication chains, syllogisms, transitivity chains, and quantifier
alternation) is documented in `report/sections/dataset.tex` under
*Synthetic*.

```powershell
python scripts/generate_fof_benchmarks.py
python tests/test_fof_generation.py
```

## Running the prover

Discover the full flag set directly from the binary; it is the authoritative
list, and the values in `config.toml` are used as defaults whenever a flag
is omitted:

```powershell
cargo run --manifest-path .\theorem_prover\Cargo.toml -- prove --help
cargo run --manifest-path .\theorem_prover\Cargo.toml -- rules --help
```

A typical run looks like:

```powershell
# Prove every problem named in a subset description, against the local TPTP copy.
cargo run --release --manifest-path .\theorem_prover\Cargo.toml -- prove `
    --tptp-root        .\TPTP-v9.2.1 `
    --subset-file      .\subset_descriptions\medium_problems.txt `
    --problem-class    provable `
    --engine           priority-id `
    --format           tsv

# Prove a directory of standalone .p files (e.g. the synthetic benchmarks).
cargo run --release --manifest-path .\theorem_prover\Cargo.toml -- prove `
    .\generated-tests `
    --problem-class    mixed `
    --engine           priority-id
```

`--problem-class` (`provable`, `unprovable`, `mixed`, `unknown`) is
required; it labels the run in the results DB so the analysis scripts can
join expected vs.\ actual.

### Engines (`--engine`)

The report evaluates three:

- `naive` — depth-first backward search (Hou's LK$'$ baseline).
- `id` — iterative deepening over the same DFS.
- `priority-id` — iterative deepening with the 6-class LK$'$ priority schedule.

See `report/sections/proposed-approach.tex` for the priority classes and
`report/sections/experimental_setup.tex` for the decided-problem counts.

### Sweeping every engine

`scripts/run_all_engines.ps1` runs `naive`, `id`, and `priority-id` in
sequence, forwarding any extra arguments to `cargo run -- prove`. This is
how each results table in the report was produced.

```powershell
.\scripts\run_all_engines.ps1 `
    --subset-file   .\subset_descriptions\medium_problems.txt `
    --tptp-root     .\TPTP-v9.2.1 `
    --problem-class provable
```

Results are appended to the SQLite file named by `results_db` in
`config.toml` (or `--persist <path>` on the CLI). One row per
`(run, problem)`.

## FOLIO

The FOLIO pipeline (download, FOL-to-TPTP export, prover invocation,
prediction TSVs) lives in `FOLIO/`. See `FOLIO/README.md`.
