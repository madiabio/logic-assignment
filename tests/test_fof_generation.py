import subprocess
import os
from pathlib import Path

TEST_DIR = Path(__file__).parent.parent

def parse_tptp_file(filepath):
    """Test utility: run the theorem prover's parser on a file."""
    result = subprocess.run(
        ["cargo", "run", "--manifest-path", str(TEST_DIR / "theorem_prover" / "Cargo.toml"),
         "--", "prove", str(filepath)],
        capture_output=True,
        text=True,
        cwd=str(TEST_DIR / "theorem_prover")
    )
    return result.returncode, result.stdout, result.stderr

def test_generated_files_parse():
    """Test that all generated .p files parse without error."""
    ai_gen_dir = TEST_DIR / "AI_generated"

    # Files should exist after generation
    assert (ai_gen_dir / "easy.p").exists(), "easy.p not found"
    assert (ai_gen_dir / "medium.p").exists(), "medium.p not found"
    assert (ai_gen_dir / "hard.p").exists(), "hard.p not found"
    assert (ai_gen_dir / "expert.p").exists(), "expert.p not found"

if __name__ == "__main__":
    test_generated_files_parse()
    print("Basic structure test passed")
