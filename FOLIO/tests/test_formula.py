import unittest

from folio.formula import UnsupportedFormula, convert_fol_formula


class TestConvertFolFormula(unittest.TestCase):
    def test_universal_implication(self):
        self.assertEqual(
            convert_fol_formula("∀x (Dog(x) → Animal(x))"),
            "(! [X] : (dog(X) => animal(X)))",
        )

    def test_existential_conjunction(self):
        self.assertEqual(
            convert_fol_formula("∃x (Dog(x) ∧ Brown(x))"),
            "(? [X] : (dog(X) & brown(X)))",
        )

    def test_nested_negation(self):
        self.assertEqual(
            convert_fol_formula("¬(Dog(a) ∨ ¬Cat(a))"),
            "~((dog(a) | ~(cat(a))))",
        )

    def test_biconditional(self):
        self.assertEqual(
            convert_fol_formula("Dog(a) ↔ Animal(a)"),
            "(dog(a) <=> animal(a))",
        )

    def test_xor_expansion(self):
        self.assertEqual(
            convert_fol_formula("Dog(a) ⊕ Cat(a)"),
            "((dog(a) | cat(a)) & ~((dog(a) & cat(a))))",
        )

    def test_equality_is_unsupported(self):
        with self.assertRaisesRegex(UnsupportedFormula, "equality"):
            convert_fol_formula("a = b")

    def test_empty_formula_raises(self):
        with self.assertRaises(UnsupportedFormula):
            convert_fol_formula("   ")

    def test_ascii_forall(self):
        # ! is the ASCII FORALL prefix; variables follow directly, no brackets in input
        self.assertEqual(
            convert_fol_formula("!x (P(x) => Q(x))"),
            "(! [X] : (p(X) => q(X)))",
        )

    def test_ascii_exists_and_not(self):
        self.assertEqual(
            convert_fol_formula("?x ~P(x)"),
            "(? [X] : ~(p(X)))",
        )

    def test_nested_quantifiers(self):
        # Inner quantifier body must be parenthesised so the parser knows where
        # the variable list ends and the body begins.
        self.assertEqual(
            convert_fol_formula("∀x (∃y (R(x, y)))"),
            "(! [X] : (? [Y] : r(X, Y)))",
        )

    def test_multiterm_atom(self):
        # Free (unbound) variables are lowercased; only quantifier-bound vars are uppercased.
        self.assertEqual(
            convert_fol_formula("P(x, y, z)"),
            "p(x, y, z)",
        )

    def test_implies_arrow_variant(self):
        self.assertEqual(
            convert_fol_formula("P(a) -> Q(a)"),
            "(p(a) => q(a))",
        )

    def test_implies_double_arrow_variant(self):
        self.assertEqual(
            convert_fol_formula("P(a) => Q(a)"),
            "(p(a) => q(a))",
        )


if __name__ == "__main__":
    unittest.main()
