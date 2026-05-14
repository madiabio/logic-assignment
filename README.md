# Logic Assignment

A Rust implementation of Hou's LK$'$ backward sequent search with two
extensions (a 6-class priority schedule and iterative deepening), evaluated
on TPTP, FOLIO, and a synthetic FOF benchmark. The calculus, engines,
dataset rationale, and results live in `report/`; this README only records
the commands that produced the committed artefacts.

## The prover

```powershell
cargo build --release --manifest-path .\theorem_prover\Cargo.toml
cargo run    --manifest-path .\theorem_prover\Cargo.toml -- prove --help
cargo run    --manifest-path .\theorem_prover\Cargo.toml -- rules --help
```

`prove --help` is the authoritative flag reference. Anything not given on
the command line falls back to `theorem_prover/config.toml`; copy
`config.toml.example` and fill in the paths. The three engines used in the
report are `naive`, `id`, and `priority-id`, selected via `--engine`.

A standalone run looks like:

```powershell
cargo run --release --manifest-path .\theorem_prover\Cargo.toml -- prove `
    --tptp-root     .\TPTP-v9.2.1 `
    --subset-file   .\subset_descriptions\medium_problems.txt `
    --engine        priority-id `
    --problem-class provable `
    --format        tsv
```

Results land in the SQLite file named by `results_db` in `config.toml`
(or `--persist <path>`), one row per `(run, problem)`.

## How the report's artefacts were generated

### TPTP subsets (`subset_descriptions/`)

Each subset file's **first line** records the exact `tptp2T` / `tptp2t`
selection command that produced it, run inside a TPTP-v9 distribution:

- `easy_problems.txt` — FOF/THM, rating 0, $\le 50$ formulae, $\le 150$ atoms, no equality or arithmetic.
- `medium_problems.txt` — FOF/THM, unbounded rating, same complexity bounds.
- `medium_problems_countersatisfiable.txt` — FOF/CSA, unbounded rating, same bounds.

To rebuild a subset, re-run the command on line 1 inside a TPTP release,
then sort/format the result with `scripts/Format_TPTP_Subset.ps1` (it
sorts by ascending atom count and can also report overlap against an
existing subset):

```powershell
.\scripts\Format_TPTP_Subset.ps1 `
    -InputFile   .\tptp_medium_countersat.txt `
    -Difficulty  medium `
    -CompareFile .\subset_descriptions\medium_problems.txt
```

### Synthetic benchmarks (`generated-tests/`)

```powershell
python scripts\generate_fof_benchmarks.py
python tests\test_fof_generation.py
```

Tier sizes and the provable/unprovable split are documented in
`generated-tests/README.md`; construction rationale is in
`report/sections/dataset.tex` under *Synthetic*.

### Engine comparison runs

`scripts/run_all_engines.ps1` invokes `prove` once per engine
(`naive`, `id`, `priority-id`), forwarding any extra flags. This produced
each comparison table in the report.

```powershell
.\scripts\run_all_engines.ps1 `
    --tptp-root     .\TPTP-v9.2.1 `
    --subset-file   .\subset_descriptions\medium_problems.txt `
    --problem-class provable
```

### FOLIO

See `FOLIO/README.md`.
