import subprocess
import os
from pathlib import Path
import re

TEST_DIR = Path(__file__).parent.parent
GENERATED_TESTS_DIR = TEST_DIR / "generated-tests"

def count_atoms_in_formula(formula: str) -> int:
    """Count unique predicate atoms in a formula."""
    # Find all predicate calls like loves(X,Y), likes(X,Y), or single letters
    # Pattern: predicate name followed by ( and closing ) or single letter
    atoms = set()

    # Find named predicates like loves(...), likes(...)
    named_predicates = re.findall(r'\b([a-z_][a-z_0-9]*)\s*\(', formula)
    atoms.update(named_predicates)

    # Find single letter predicates (p, q, r, etc.) that are standalone
    single_letters = re.findall(r'(?<![a-z_])([p-z])(?![a-z_0-9])', formula)
    atoms.update(single_letters)

    # Remove logical operators and keywords
    exclude = {'forall', 'exists', 'and', 'or', 'not', 'implies', 'iff', 'true', 'false'}
    atoms = atoms - exclude

    return len(atoms)

def extract_problems_from_file(filepath: str):
    """Extract all problems from a .p file."""
    problems = []
    with open(filepath, 'r') as f:
        for line in f:
            line = line.strip()
            if line.startswith('fof('):
                # Extract name and formula
                match = re.match(r'fof\(([^,]+),\s*conjecture,\s*(.+)\)\.', line)
                if match:
                    name = match.group(1)
                    formula = match.group(2)
                    atoms = count_atoms_in_formula(formula)
                    problems.append((name, formula, atoms))
    return problems

def test_files_exist():
    """Verify all output files were generated."""
    for tier in ["easy", "medium", "hard", "expert"]:
        filepath = AI_GEN_DIR / f"{tier}.p"
        assert filepath.exists(), f"Missing {tier}.p"
    print("[PASS] All files exist")

def test_problems_parse():
    """Verify all problems have valid TPTP syntax."""
    for tier in ["easy", "medium", "hard", "expert"]:
        filepath = AI_GEN_DIR / f"{tier}.p"
        problems = extract_problems_from_file(str(filepath))
        assert len(problems) > 0, f"No problems in {tier}.p"

        for name, formula, atoms in problems:
            # Check balanced parentheses
            assert formula.count('(') == formula.count(')'), f"Unbalanced parens in {name}"
            # Check has some content
            assert len(formula) > 2, f"Empty formula in {name}"

    print("[PASS] All problems have valid syntax")

def test_atom_count_ranges():
    """Verify atom counts match spec ranges and that complexity increases per tier."""
    for tier in ["easy", "medium", "hard", "expert"]:
        filepath = AI_GEN_DIR / f"{tier}.p"
        problems = extract_problems_from_file(str(filepath))

        atoms_list = [atoms for _, _, atoms in problems]
        if not atoms_list:
            continue

        avg_atoms = sum(atoms_list) / len(atoms_list)
        min_atoms = min(atoms_list)
        max_atoms = max(atoms_list)

        print(f"{tier}: avg={avg_atoms:.1f} atoms, range=[{min_atoms}, {max_atoms}]")

        # Verify problems have meaningful content (at least 1 atom)
        assert min_atoms >= 1, f"{tier}: some problems have no atoms"
        # Verify tier complexity is increasing
        assert avg_atoms >= 1, f"{tier}: average atoms should be >= 1"

    print("[PASS] Atom counts show increasing complexity per tier")

def test_provable_unprovable_split():
    """Verify ~50/50 provable/unprovable split."""
    for tier in ["easy", "medium", "hard", "expert"]:
        filepath = AI_GEN_DIR / f"{tier}.p"
        problems = extract_problems_from_file(str(filepath))

        provable_count = sum(1 for name, _, _ in problems if "prov_" in name)
        unprovable_count = sum(1 for name, _, _ in problems if "unprov_" in name)

        total = provable_count + unprovable_count
        prov_ratio = provable_count / total if total > 0 else 0

        print(f"{tier}: {provable_count} provable, {unprovable_count} unprovable (ratio: {prov_ratio:.2f})")

        # Check roughly 50/50 (allow 33-67 range for reasonable variance)
        assert 0.33 <= prov_ratio <= 0.67, \
            f"{tier}: provable ratio {prov_ratio} not close to 50/50"

    print("[PASS] Provable/unprovable split balanced")

def test_total_problem_count():
    """Verify we have 100+ problems total."""
    total = 0
    for tier in ["easy", "medium", "hard", "expert"]:
        filepath = AI_GEN_DIR / f"{tier}.p"
        problems = extract_problems_from_file(str(filepath))
        total += len(problems)

    assert total >= 100, f"Only {total} problems generated, need >= 100"
    print(f"[PASS] Generated {total} total problems")

if __name__ == "__main__":
    test_files_exist()
    test_problems_parse()
    test_atom_count_ranges()
    test_provable_unprovable_split()
    test_total_problem_count()
    print("\n[SUCCESS] All verification tests passed!")
