from __future__ import annotations

import pathlib
import subprocess
from datetime import datetime, timezone


def run_prover(
    out_dir: pathlib.Path,
    db_path: pathlib.Path,
    timeout_ms: int,
    engine: str,
) -> subprocess.CompletedProcess:
    db_path.parent.mkdir(parents=True, exist_ok=True)
    # folio/folio/prover.py → folio/folio/ → folio/ → repo root
    repo_root = pathlib.Path(__file__).resolve().parents[2]
    command = [
        "cargo",
        "run",
        "--quiet",
        "--manifest-path",
        str(repo_root / "theorem_prover" / "Cargo.toml"),
        "--",
        "prove",
        "--problem-class",
        "mixed",
        "--format",
        "tsv",
        "--timeout-ms",
        str(timeout_ms),
        "--engine",
        engine,
        "--persist",
        str(db_path),
        "--run-label",
        f"folio-{datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ')}",
        str(out_dir),
    ]
    return subprocess.run(command, check=False, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
