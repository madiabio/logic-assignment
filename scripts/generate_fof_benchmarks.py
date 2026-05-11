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

class MediumTierProblems:
    """Generate medium tier problems (universal quantifiers, 20-50 atoms)."""

    def __init__(self, seed: int = None):
        self.seed = seed
        self.problem_counter = 0

    def provable_medium(self) -> Tuple[str, str]:
        """Generate a provable medium problem with universal quantifiers."""
        self.problem_counter += 1

        # Pattern: Universal instantiation
        # ! [X] : p(X) is provable if we construct the right instances
        if self.problem_counter % 2 == 0:
            gen = QuantifiedFormulaGenerator(seed=self.seed)
            p_univ = gen.universal_formula(gen.simple_predicate())
            # Create another instance: p is true everywhere, so it's true for specific X
            formula = f"({p_univ} => {gen.simple_predicate()})"
            return (f"medium_prov_{self.problem_counter}", formula)

        # Pattern: Tautology with quantifiers
        # ! [X] : (p(X) | ~p(X)) is always true
        else:
            gen = QuantifiedFormulaGenerator(seed=self.seed)
            var = gen.fresh_var()
            pred = gen.fresh_pred(1)
            body = f"({pred}({var}) | ~{pred}({var}))"
            formula = gen.universal_formula(body)
            return (f"medium_prov_{self.problem_counter}", formula)

    def unprovable_medium(self) -> Tuple[str, str]:
        """Generate an unprovable medium problem."""
        self.problem_counter += 1

        # Pattern: Existential claim without proof
        # ? [X] : p(X) is unprovable without premises asserting it
        if self.problem_counter % 2 == 0:
            gen = QuantifiedFormulaGenerator(seed=self.seed)
            formula = gen.existential_formula(gen.simple_predicate())
            return (f"medium_unprov_{self.problem_counter}", formula)

        # Pattern: Broken deduction chain
        # ! [X] : (p(X) => q(X)), and we need to prove ! [X] : r(X)
        # but r is unrelated to p and q
        else:
            gen = QuantifiedFormulaGenerator(seed=self.seed)
            var = gen.fresh_var()
            p_pred = gen.fresh_pred(1)
            q_pred = gen.fresh_pred(1)
            r_pred = gen.fresh_pred(1)

            premise = f"! [X] : ({p_pred}(X) => {q_pred}(X))"
            goal = f"! [X] : {r_pred}(X)"
            formula = f"({premise} => {goal})"
            return (f"medium_unprov_{self.problem_counter}", formula)

class HardTierProblems:
    """Generate hard tier problems (nested quantifiers, 50-100 atoms)."""

    def __init__(self, seed: int = None):
        self.seed = seed
        self.problem_counter = 0

    def provable_hard(self) -> Tuple[str, str]:
        """Generate a provable hard problem with nested quantifiers."""
        self.problem_counter += 1

        # Pattern: Nested universal quantifiers with implication
        # ! [X] : ! [Y] : (p(X,Y) => q(X,Y))
        # This is provable because it's a valid form
        if self.problem_counter % 2 == 0:
            gen = QuantifiedFormulaGenerator(seed=self.seed)
            x = gen.fresh_var()
            y = gen.fresh_var()
            p_pred = gen.fresh_pred(2)
            q_pred = gen.fresh_pred(2)
            body = f"({p_pred}({x},{y}) => {q_pred}({x},{y}))"
            inner = f"! [{y}] : ({body})"
            formula = f"! [{x}] : ({inner})"
            return (f"hard_prov_{self.problem_counter}", formula)

        # Pattern: Tautology with multiple quantifiers
        else:
            gen = QuantifiedFormulaGenerator(seed=self.seed)
            x = gen.fresh_var()
            y = gen.fresh_var()
            p = gen.fresh_pred(2)
            body = f"({p}({x},{y}) | ~{p}({x},{y}))"
            inner = f"! [{y}] : ({body})"
            formula = f"! [{x}] : ({inner})"
            return (f"hard_prov_{self.problem_counter}", formula)

    def unprovable_hard(self) -> Tuple[str, str]:
        """Generate an unprovable hard problem."""
        self.problem_counter += 1

        # Pattern: Impossible existential in nested context
        # ! [X] : ? [Y] : p(X,Y) then prove ! [X] : ! [Y] : q(X,Y) - unrelated
        if self.problem_counter % 2 == 0:
            gen = QuantifiedFormulaGenerator(seed=self.seed)
            x = gen.fresh_var()
            y = gen.fresh_var()
            p = gen.fresh_pred(2)
            q = gen.fresh_pred(2)

            premise = f"! [{x}] : ? [{y}] : {p}({x},{y})"
            goal = f"! [{x}] : ! [{y}] : {q}({x},{y})"
            formula = f"({premise} => {goal})"
            return (f"hard_unprov_{self.problem_counter}", formula)

        # Pattern: Conflicting constraints
        else:
            gen = QuantifiedFormulaGenerator(seed=self.seed)
            x = gen.fresh_var()
            y = gen.fresh_var()
            p = gen.fresh_pred(2)
            q = gen.fresh_pred(2)
            r = gen.fresh_pred(2)

            # p and ~p in different contexts can't both be proven
            body1 = f"({p}({x},{y}) => {q}({x},{y}))"
            body2 = f"(~{p}({x},{y}) => {r}({x},{y}))"
            formula = f"! [{x}] : ! [{y}] : (({body1}) & ({body2}))"
            return (f"hard_unprov_{self.problem_counter}", formula)

class ExpertTierProblems:
    """Generate expert tier problems (deeply nested, 100-150 atoms)."""

    def __init__(self, seed: int = None):
        self.seed = seed
        self.problem_counter = 0

    def provable_expert(self) -> Tuple[str, str]:
        """Generate a provable expert problem with deep nesting."""
        self.problem_counter += 1

        # Pattern: Triple nested quantifiers with tautology
        if self.problem_counter % 2 == 0:
            gen = QuantifiedFormulaGenerator(seed=self.seed)
            x = gen.fresh_var()
            y = gen.fresh_var()
            z = gen.fresh_var()
            p = gen.fresh_pred(3)

            inner = f"({p}({x},{y},{z}) | ~{p}({x},{y},{z}))"
            mid = f"? [{z}] : ({inner})"
            outer = f"! [{y}] : ({mid})"
            formula = f"! [{x}] : ({outer})"
            return (f"expert_prov_{self.problem_counter}", formula)

        # Pattern: Provable chain of implications
        else:
            gen = QuantifiedFormulaGenerator(seed=self.seed)
            x = gen.fresh_var()
            y = gen.fresh_var()
            a = gen.fresh_pred(2)
            b = gen.fresh_pred(2)
            c = gen.fresh_pred(2)

            # (p=>q) & (q=>r) => (p=>r)
            body = f"(({a}({x},{y}) => {b}({x},{y})) & ({b}({x},{y}) => {c}({x},{y}))) => ({a}({x},{y}) => {c}({x},{y}))"
            formula = f"! [{x}] : ! [{y}] : ({body})"
            return (f"expert_prov_{self.problem_counter}", formula)

    def unprovable_expert(self) -> Tuple[str, str]:
        """Generate an unprovable expert problem."""
        self.problem_counter += 1

        # Pattern: Deeply nested with unrelated predicates
        if self.problem_counter % 2 == 0:
            gen = QuantifiedFormulaGenerator(seed=self.seed)
            x = gen.fresh_var()
            y = gen.fresh_var()
            z = gen.fresh_var()
            p = gen.fresh_pred(3)
            q = gen.fresh_pred(3)

            premise = f"! [{x}] : ! [{y}] : ? [{z}] : {p}({x},{y},{z})"
            goal = f"! [{x}] : ! [{y}] : ! [{z}] : {q}({x},{y},{z})"
            formula = f"({premise} => {goal})"
            return (f"expert_unprov_{self.problem_counter}", formula)

        # Pattern: Contradictory constraints at multiple levels
        else:
            gen = QuantifiedFormulaGenerator(seed=self.seed)
            x = gen.fresh_var()
            y = gen.fresh_var()
            z = gen.fresh_var()
            p = gen.fresh_pred(3)
            q = gen.fresh_pred(3)
            r = gen.fresh_pred(3)

            clause1 = f"({p}({x},{y},{z}) => {q}({x},{y},{z}))"
            clause2 = f"(~{p}({x},{y},{z}) => {r}({x},{y},{z}))"
            goal = f"~{q}({x},{y},{z})"

            formula = f"! [{x}] : ! [{y}] : ? [{z}] : (({clause1} & {clause2}) => {goal})"
            return (f"expert_unprov_{self.problem_counter}", formula)

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

    medium_gen = MediumTierProblems(seed=42)

    # Generate 5 provable
    for _ in range(5):
        name, formula = medium_gen.provable_medium()
        atom_count = AtomCounter.count_atoms(formula)
        print(f"{name}: atoms={atom_count}")

    # Generate 5 unprovable
    for _ in range(5):
        name, formula = medium_gen.unprovable_medium()
        atom_count = AtomCounter.count_atoms(formula)
        print(f"{name}: atoms={atom_count}")

    hard_gen = HardTierProblems(seed=42)
    expert_gen = ExpertTierProblems(seed=42)

    for _ in range(3):
        name, formula = hard_gen.provable_hard()
        print(f"{name}: atoms={AtomCounter.count_atoms(formula)}")

    for _ in range(3):
        name, formula = expert_gen.provable_expert()
        print(f"{name}: atoms={AtomCounter.count_atoms(formula)}")
