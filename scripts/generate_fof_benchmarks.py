#!/usr/bin/env python3
"""
FOF Benchmark Problem Generator

Generates TPTP FOF problems of varying difficulty for benchmarking
the theorem prover. Problems are organized into four tiers (easy, medium,
hard, expert) with 50/50 provable/unprovable split.
"""

import random
from typing import Set, Tuple

class AtomCounter:
    """Counts unique atoms in a formula string."""

    @staticmethod
    def count_atoms(formula: str) -> int:
        """Count unique predicate atoms in a formula."""
        # Remove comments, whitespace, operators
        import re
        # Extract predicate names (uppercase letters followed by optional args)
        atoms = re.findall(r'\b[a-z_][a-z_0-9]*(?:\([^)]*\))?', formula)
        # Remove quantifier variables and operators
        atoms = [a for a in atoms if a not in ['forall', 'exists', 'and', 'or', 'not', 'implies', 'iff']]
        return len(set(atoms))

class PropositionGenerator:
    """Generates simple propositional formulas."""

    def __init__(self, seed: int = None):
        if seed is not None:
            random.seed(seed)
        self.prop_counter = 0

    def fresh_prop(self) -> str:
        """Generate a fresh proposition name (p, q, r, ..., p1, p2, ...)."""
        if self.prop_counter < 26:
            char = chr(ord('p') + self.prop_counter)
            self.prop_counter += 1
            return char
        else:
            self.prop_counter += 1
            return f"p{self.prop_counter}"

    def tautology(self) -> str:
        """Generate a tautology: p | ~p."""
        p = self.fresh_prop()
        return f"({p} | ~{p})"

    def simple_proposition(self) -> str:
        """Generate a single proposition."""
        return self.fresh_prop()

    def conjunction(self, count: int = 2) -> str:
        """Generate a conjunction of n propositions."""
        props = [self.fresh_prop() for _ in range(count)]
        return "(" + " & ".join(props) + ")"

    def disjunction(self, count: int = 2) -> str:
        """Generate a disjunction of n propositions."""
        props = [self.fresh_prop() for _ in range(count)]
        return "(" + " | ".join(props) + ")"

class QuantifiedFormulaGenerator:
    """Generates first-order formulas with quantifiers."""

    def __init__(self, seed: int = None):
        if seed is not None:
            random.seed(seed)
        self.var_counter = 0
        self.pred_counter = 0

    def fresh_var(self) -> str:
        """Generate fresh variables X, Y, Z, U, V, W."""
        vars_list = ['X', 'Y', 'Z', 'U', 'V', 'W']
        if self.var_counter < len(vars_list):
            var = vars_list[self.var_counter]
            self.var_counter += 1
            return var
        else:
            self.var_counter += 1
            return f"V{self.var_counter}"

    def fresh_pred(self, arity: int = 1) -> str:
        """Generate fresh predicate names: p, q, r, likes, loves, etc."""
        unary_preds = ['mortal', 'human', 'student', 'teacher']
        binary_preds = ['loves', 'likes', 'knows', 'parent_of']
        ternary_preds = ['teaches', 'between', 'related']

        if arity == 1:
            preds = unary_preds
        elif arity == 2:
            preds = binary_preds
        else:
            preds = ternary_preds

        if self.pred_counter < len(preds):
            pred = preds[self.pred_counter]
        else:
            pred = f"p{self.pred_counter}"

        self.pred_counter += 1
        return pred

    def universal_formula(self, body: str) -> str:
        """Wrap a formula in universal quantifier: ! [X] : body."""
        var = self.fresh_var()
        return f"! [{var}] : ({body})"

    def existential_formula(self, body: str) -> str:
        """Wrap a formula in existential quantifier: ? [X] : body."""
        var = self.fresh_var()
        return f"? [{var}] : ({body})"

    def simple_predicate(self, var: str = None, arity: int = 1) -> str:
        """Generate a predicate applied to variables."""
        if var is None:
            var = self.fresh_var()

        pred = self.fresh_pred(arity)

        if arity == 1:
            return f"{pred}({var})"
        elif arity == 2:
            var2 = self.fresh_var()
            return f"{pred}({var}, {var2})"
        else:  # arity == 3
            var2 = self.fresh_var()
            var3 = self.fresh_var()
            return f"{pred}({var}, {var2}, {var3})"

    def implication(self, antecedent: str, consequent: str) -> str:
        """Create an implication: antecedent => consequent."""
        return f"({antecedent} => {consequent})"

class EasyTierProblems:
    """Generate easy tier problems (propositional, 5-20 atoms)."""

    def __init__(self, seed: int = None):
        self.seed = seed
        self.problem_counter = 0

    def provable_easy(self) -> Tuple[str, str]:
        """Generate a provable easy problem.

        Returns: (problem_name, formula_string)
        """
        self.problem_counter += 1

        # Pattern: Tautology (p | ~p)
        if self.problem_counter % 3 == 0:
            gen = PropositionGenerator(seed=self.seed)
            formula = gen.tautology()
            return (f"easy_prov_{self.problem_counter}", formula)

        # Pattern: Modus ponens style (p, (p => q) implies q)
        # Represented as: (p & (p => q)) => q
        elif self.problem_counter % 3 == 1:
            gen = PropositionGenerator(seed=self.seed)
            p = gen.fresh_prop()
            q = gen.fresh_prop()
            # This is valid: if you have p and p=>q, you can derive q
            formula = f"((({p} & ({p} => {q})) => {q}))"
            return (f"easy_prov_{self.problem_counter}", formula)

        # Pattern: De Morgan's laws
        else:
            gen = PropositionGenerator(seed=self.seed)
            p = gen.fresh_prop()
            q = gen.fresh_prop()
            # ~(p & q) is equivalent to (~p | ~q), so this is a tautology
            formula = f"(~({p} & {q}) | (~{p} | ~{q}))"
            return (f"easy_prov_{self.problem_counter}", formula)

    def unprovable_easy(self) -> Tuple[str, str]:
        """Generate an unprovable easy problem.

        Returns: (problem_name, formula_string)
        """
        self.problem_counter += 1

        # Pattern: Independent propositions (p, q, goal=r, where r is unrelated)
        if self.problem_counter % 2 == 0:
            gen = PropositionGenerator(seed=self.seed)
            p = gen.fresh_prop()
            q = gen.fresh_prop()
            r = gen.fresh_prop()
            # No way to derive r from p and q
            formula = f"({p} & {q}) => {r}"
            return (f"easy_unprov_{self.problem_counter}", formula)

        # Pattern: Contradiction that doesn't resolve
        else:
            gen = PropositionGenerator(seed=self.seed)
            p = gen.fresh_prop()
            q = gen.fresh_prop()
            # p & ~p is unprovable (contradictory premises don't prove anything useful)
            formula = f"({p} & ~{p}) => {q}"
            return (f"easy_unprov_{self.problem_counter}", formula)

def main():
    pass

if __name__ == "__main__":
    easy_gen = EasyTierProblems(seed=42)

    # Generate 5 provable
    for _ in range(5):
        name, formula = easy_gen.provable_easy()
        atom_count = AtomCounter.count_atoms(formula)
        print(f"{name}: {formula} (atoms: {atom_count})")

    # Generate 5 unprovable
    for _ in range(5):
        name, formula = easy_gen.unprovable_easy()
        atom_count = AtomCounter.count_atoms(formula)
        print(f"{name}: {formula} (atoms: {atom_count})")
