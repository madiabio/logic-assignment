# `theorem_prover/`

A first-order logic prover implementing Hou's LK′ backward sequent
search (`naive`), an iterative-deepening envelope around it (`id`), and a
6-class priority-scheduled variant of iterative deepening (`priority-id`).
All three engines share the parser, sequent representation, rule layer,
and `backwards_search` kernel; only the scheduler differs.

The calculus, engine designs, and evaluation are documented in the
project report:

- `report/sections/proposed-approach.tex` — system architecture, the
  LK′ baseline, the priority schedule (six classes), and the
  iterative-deepening envelope.
- `report/sections/implementation.tex` — Rust crates, run configuration,
  and hardware.

This README only covers building, running, and inspecting the binary.
The CLI's own `--help` output is the authoritative flag reference.

## Build and run

The prover **must be run from inside `theorem_prover/`**: the committed
`config.toml` resolves `tptp_root`, `default_subset_file`, and `results_db`
relative to this directory.

```powershell
cd theorem_prover
copy config.toml.example config.toml     # then edit paths and limits
cargo build --release
cargo run -- prove --help                # authoritative flag reference
cargo run -- rules --help
```

A typical proof run against a subset description:

```powershell
cargo run --release -- prove `
    --subset-file   ..\subset_descriptions\medium_problems.txt `
    --engine        priority-id `
    --problem-class provable `
    --format        tsv
```

A typical run against a directory of standalone `.p` files:

```powershell
cargo run --release -- prove `
    ..\generated-tests `
    --engine        priority-id `
    --problem-class mixed
```

Every flag has a default in `config.toml`, so a fully configured run
collapses to `cargo run --release -- prove --engine priority-id
--problem-class provable`. `--subset-file` and `--tptp-root` must either
both be passed on the CLI or both come from `config.toml`; the prover
errors out if only one is supplied.

## Subcommands

| Command | Purpose                                                                  |
| ------- | ------------------------------------------------------------------------ |
| `prove` | Run proof search over a `.p` file, a directory of `.p` files, or a subset description against a TPTP root. |
| `rules` | Inspect which LK′ rules apply to each formula in the input, without running search. Useful for debugging the parser or the rule matcher. |

## Engines

Selected via `--engine`:

- `naive` — Hou's depth-first backward search. Baseline.
- `id` — iterative deepening over the same DFS: depth limits 1, 2, 3, …
  up to `--max-depth`, restarting between iterations.
- `priority-id` — iterative deepening with the 6-class priority schedule
  (closing rules first, then non-branching propositional, then branching,
  then eigenvariable, then reusable quantifier, then identity reuse).

Per-problem result statuses written to the DB and TSV output:

| Status            | Meaning                                                       |
| ----------------- | ------------------------------------------------------------- |
| `provable`        | All branches closed by closing rules.                         |
| `not_provable`    | Every branch is unclosable. Sound only when no resource bound triggered. |
| `unknown`         | A budget (`max_depth`, `max_steps`, `max_fresh_terms_per_quantifier`) was hit; `unknown_reason` records which. |
| `timeout`         | Wall-clock `timeout_ms` elapsed.                              |
| `cancelled`       | Ctrl-C from the user.                                         |
| `not_implemented` | Encountered a TPTP feature the parser does not yet support.   |
| `error`           | Internal error.                                               |

## Configuration

`config.toml` holds the defaults for every CLI flag. The committed file
mirrors the run configuration cited in the report:

| Setting                            | Report value                                 |
| ---------------------------------- | -------------------------------------------- |
| `timeout_ms`                       | `1000`                                       |
| `max_depth`                        | `128`                                        |
| `max_steps`                        | `10000` (TPTP/FOLIO); raise for synthetic    |
| `max_biconditionals`               | `6` (problems above this skip parsing)       |
| `max_fresh_terms_per_quantifier`   | `1`                                          |
| `tptp_root`                        | `..\TPTP-v9.2.1`                             |
| `default_subset_file`              | `..\subset_descriptions\medium_problems_countersatisfiable.txt` |
| `results_db`                       | `..\results.db`                              |

Copy `config.toml.example` and fill it in; the real `config.toml` is
gitignored so each checkout can carry its own paths and limits.

## Persistence

`--persist <path>` (default: `results_db` from `config.toml`) writes to a
SQLite database with two tables:

- **`runs`** — one row per invocation: `label`, `timestamp`, `engine`,
  `timeout_ms`, `max_depth`, `max_steps`, `max_fresh_terms_per_quantifier`,
  `problem_class`.
- **`results`** — one row per `(run, problem)`: `problem_id`, `path`,
  `status`, `elapsed_ms`, `formulae`, `atoms`, and `unknown_reason` when
  the status is `unknown`.

Pass `--persist false` to disable persistence (useful for one-off
debugging runs).

## Source layout

```
src/
├── main.rs              entrypoint, wires CLI into the pipeline
├── lib.rs               re-exports for integration tests
├── pipeline.rs          orchestrates parse → prove → persist for one problem
├── cli/                 clap definitions, config loading, subcommand dispatch
│   ├── args/            CliOptions, ProveCommand, RulesCommand, engine enum
│   ├── prove.rs         `prove` subcommand entry
│   ├── rules.rs         `rules` subcommand entry
│   ├── subset.rs        load and iterate a subset description file
│   ├── run.rs           per-problem driver, timing, cancellation
│   ├── output.rs        human / TSV formatters
│   └── config.rs        merge CLI flags with config.toml defaults
├── parser/              pest grammar (`tptp.pest`) and AST builder
├── ast/                 Formula and Term definitions
├── proof/
│   ├── sequent.rs       Γ ⊢ Δ representation
│   ├── prover.rs        top-level entrypoint into search
│   ├── rules/           rule kinds, matcher, and apply implementations
│   ├── apply.rs         glue between matched rules and the kernel
│   ├── quantifier/      fresh constants, instantiation, witness budget
│   ├── search/          backwards_search kernel
│   │   ├── scheduler.rs the 6-class LK′ priority order
│   │   ├── branch_state.rs   per-branch term store and quantifier reuse
│   │   └── engine/      `naive.rs`, `iterative_deepening.rs`
│   └── defaults.rs      proof-option defaults
└── persistence/         SQLite schema, run/result writes, read helpers
```

## Tests

```powershell
cd theorem_prover
cargo test --release
```

Integration tests in `tests/` cover the parser (`parser_tests.rs`,
`ast_tests.rs`), rule application (`apply_rule_tests.rs`,
`proof_rules_tests.rs`), the priority scheduler
(`priority_engine_tests.rs`, `scheduler_lk_priority_tests.rs`), the
proof kernel (`prover_tests.rs`, `pipeline_tests.rs`,
`sequent_tests.rs`), and the CLI surface (`cli_include_tests.rs`,
`cli_rules_tests.rs`, `include_loader_tests.rs`).
