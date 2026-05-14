# `scripts/`

Three scripts that produced the report's benchmark inputs and result tables.

## `generate_fof_benchmarks.py` — synthetic FOF generator

Produces the 202-problem synthetic benchmark cited in
`report/sections/dataset.tex` (*Synthetic*) and
`report/sections/experimental_setup.tex`. Run with:

```powershell
python scripts/generate_fof_benchmarks.py
```

Output: one `.p` file per problem in `generated-tests/` (the default; pass
a different path to `main()` to override). The script clears any existing
`.p` files in the output directory before writing, so it is idempotent.

### Problem families

Each family ships in provable and unprovable variants. The unprovable
variants are genuinely *CounterSatisfiable* (a model refutes the
conjecture), not negated tautologies, so a sound prover can decide them.

| Family | Provable form | Unprovable form |
| --- | --- | --- |
| **Implication chain** (`chain_provable`, `chain_unprovable`) | `p0(a)`, $n$ steps `∀X. p_i(X) ⇒ p_{i+1}(X)`, plus $d$ distractor chains `q_i ⇒ q_{i+1}`. Goal: `p_n(a)`. | Same shape with the step at position `gap` removed; the chain is severed and the goal is unreachable. |
| **Syllogism** (`syllogism_provable`, `syllogism_unprovable`) | A `depth+1`-deep class hierarchy `c_i(X) ⇒ c_{i+1}(X)` plus `c_0(socrates)`. Goal: `c_{depth+1}(socrates)`. | Same hierarchy and ground fact, but the goal asks about `plato` instead. |
| **Transitivity chain** (`trans_provable`, `trans_unprovable`) | One `rel` transitivity axiom plus $n$ ground edges `rel(a_i, a_{i+1})`, with optional unrelated `other(b_i, b_{i+1})` distractors. Goal: `rel(a_0, a_n)`. | Same axiom and edges, but the goal asks for `rel(a_0, a_{n+2})`, beyond the chain's reach. |
| **Quantifier alternation** (`q_*` helpers) | Single-formula classical theorems: drinker, converse Barcan, Pelletier 18, Russell diagonal, etc. | Single-formula non-theorems: Barcan, `∃ ⇒ ∀`, contradiction-everywhere, and similar. |

### Tier construction

`collect_all()` calls the family helpers with widening parameters to fill
each tier. Approximate sizes and parameter ranges:

| Tier   | Total | Provable | Unprovable | Chain length | Trans steps | Distractors |
| ------ | -----:| --------:| ----------:| ------------ | ----------- | ----------- |
| easy   |   ~32 |      ~18 |        ~14 | 2--3         | 2--3        | 0--2        |
| medium |   ~60 |      ~30 |        ~30 | 4--6         | 4--6        | 2--5        |
| hard   |   ~60 |      ~30 |        ~30 | 7--10        | 7--10       | 4--10       |
| expert |   ~50 |      ~25 |        ~25 | 11--15       | 10--13      | 6--14       |

Run `python tests/test_fof_generation.py` to re-verify the generator.

### Why these problems discriminate between engines

Each family targets a different engine weakness (see the docstring at the
top of `generate_fof_benchmarks.py`, and `report/sections/dataset.tex`):

- **Chain + distractors** rewards priority scheduling, which ranks
  productive rule applications over distractor steps.
- **Transitivity chains** reward iterative deepening, which avoids
  over-committing to one branch when the same axiom must be reused.
- **Unprovable chains/syllogisms** test whether the prover can recognise
  CounterSatisfiable rather than time out on a non-existent proof.

## `Format_TPTP_Subset.ps1` — TPTP subset reformatter

Takes a raw `tptp2t` listing, sorts by ascending atom count (problem name
as tiebreaker), and re-emits the file with the canonical comment header.

```powershell
.\scripts\Format_TPTP_Subset.ps1 `
    -InputFile   .\tptp_medium_countersat.txt `
    -Difficulty  medium `
    -CompareFile .\subset_descriptions\medium_problems.txt
```

`-CompareFile` is optional; when supplied, the script lists problems that
appear in both files, which is how the medium-THM and medium-CSA subsets
were checked for disjointness. Output is written to
`{Difficulty}_problems_countersatisfiable.txt`.

## `run_all_engines.ps1` — engine sweep

Runs `cargo run prove` once per engine (`naive`, `id`, `priority-id`) and
forwards any extra arguments to each invocation. This produced every
per-engine results table in the report.

```powershell
.\scripts\run_all_engines.ps1 `
    --tptp-root     .\TPTP-v9.2.1 `
    --subset-file   .\subset_descriptions\medium_problems.txt `
    --problem-class provable
```

The script aborts the sweep if any engine returns a non-zero exit code.
