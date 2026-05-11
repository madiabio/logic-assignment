# FOF Benchmark Problem Generation Design

**Date:** 2026-05-11  
**Objective:** Generate 100+ TPTP FOF problems organized by difficulty for benchmarking theorem prover performance

## Overview

Create a suite of first-order logic problems in TPTP FOF format to benchmark the theorem prover's performance across varying complexity levels. Problems are organized into a dedicated `AI_generated/` directory with separate `.p` files by difficulty tier.

## Problem Specification

### Distribution & Coverage
- **Total:** 100+ problems across all tiers
- **Split:** 50/50 provable vs unprovable
- **Atom count range:** 0-150 atoms per problem
- **Output location:** `AI_generated/` directory (new)
- **File format:** TPTP `.p` files (one per difficulty tier)

### Difficulty Tiers

#### Easy Tier (`easy.p`)
- **Atom count:** 5-20
- **Logic:** Propositional (no quantifiers)
- **Structures:** Simple conjunctions and disjunctions
- **Predicates:** 2-4 unary predicates
- **Examples:** `p | q`, `(p & q) | r`, `~p | ~q`
- **Count:** ~15 problems (8 provable, 7 unprovable)

#### Medium Tier (`medium.p`)
- **Atom count:** 20-50
- **Logic:** Universal quantifiers, first-order logic
- **Structures:** Implications, conjunctions with quantifiers
- **Predicates:** 3-6 predicates with 1-2 arity
- **Examples:** `! [X] : (p(X) => q(X))`, `! [X] : p(X) & ! [Y] : q(Y)`
- **Count:** ~30 problems (15 provable, 15 unprovable)

#### Hard Tier (`hard.p`)
- **Atom count:** 50-100
- **Logic:** Nested quantifiers (∀∃, ∃∀ combinations)
- **Structures:** Complex nesting, multiple predicates
- **Predicates:** 4-8 predicates with 1-3 arity (e.g., `p(X)`, `likes(X,Y)`, `teaches(X,Y,Z)`)
- **Examples:** `! [X] : ? [Y] : (p(X) => likes(X,Y))`, multi-clause implications
- **Count:** ~30 problems (15 provable, 15 unprovable)

#### Expert Tier (`expert.p`)
- **Atom count:** 100-150
- **Logic:** Deeply nested quantifiers, complex constraints
- **Structures:** Mixed ∀/∃ nesting, 3+ levels of quantification
- **Predicates:** 6-10 predicates with varying arities
- **Count:** ~25 problems (13 provable, 12 unprovable)

### Provable Problem Generation

Provable formulas are constructed using patterns that guarantee logical validity:

1. **Tautologies:** `p | ~p`, law of excluded middle
2. **Valid deductions:** If premises logically entail conclusion (modus ponens, transitivity)
3. **Contradictions resolving to goal:** Premises that contradict negation of goal
4. **Universal instantiation:** `! [X] : p(X)` with specific instances

### Unprovable Problem Generation

Unprovable formulas are constructed to fail proof:

1. **Independent propositions:** No logical relationship between premises and goal
2. **Broken deduction chains:** Missing a required premise for valid inference
3. **Contradictory constraints:** Inconsistent premises that don't entail the goal
4. **Uninstantiable quantifiers:** Existential claims with no matching premises

## TPTP FOF Format

Each problem uses standard TPTP FOF syntax:

```
fof(name, conjecture, formula).
```

Where:
- `name`: Unique identifier (e.g., `easy_1`, `medium_tautology_5`)
- `conjecture`: Problem type (always `conjecture` for our benchmark)
- `formula`: First-order logic formula with connectives `&`, `|`, `~`, `=>`, `<=>`, `!`, `?`

Example:
```
fof(easy_1, conjecture, p | ~p).
fof(medium_5, conjecture, ! [X] : (p(X) => q(X))).
```

## Variable & Predicate Naming

- **Variables:** Standard first-order: `X`, `Y`, `Z`, `U`, `V`, `W`
- **Predicates:** Meaningful English names for readability
  - Unary: `p`, `q`, `r`, `likes`, `mortal`, `human`
  - Binary: `loves`, `parent_of`, `knows`, `teaches`
  - Ternary: `teaches`, `between`, `related`

## File Organization

```
AI_generated/
├── easy.p       (~15 problems)
├── medium.p     (~30 problems)
├── hard.p       (~30 problems)
└── expert.p     (~25 problems)
```

## Success Criteria

1. All files parse correctly with the theorem prover's `parse_tptp()` function
2. Provable problems prove true; unprovable problems time out or return false
3. Difficulty progression is observable (easy solves faster than expert)
4. Atom count distribution matches specification (0-150 range)
5. 50/50 provable/unprovable split maintained across all tiers
