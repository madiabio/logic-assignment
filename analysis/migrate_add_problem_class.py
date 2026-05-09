#!/usr/bin/env python3
"""
One-time migration script to add problem_class column to the runs table.

This script:
1. Opens results.db (or accepts --db arg for path)
2. Adds a problem_class TEXT column to runs table (idempotent)
3. Sets all existing rows to 'provable'
4. Prints a confirmation table with run_id, engine, label, problem_class
"""

from __future__ import annotations

import argparse
import sqlite3
from pathlib import Path

import pandas as pd


def resolve_db_path(db_arg: str | None = None) -> Path:
    """Resolve the path to results.db, with optional override via --db argument."""
    if db_arg:
        db_path = Path(db_arg)
        if not db_path.exists():
            raise FileNotFoundError(f"Database not found at {db_path}")
        return db_path

    base_dir = Path(__file__).resolve().parent
    candidates = [base_dir / "results.db", base_dir.parent / "results.db"]

    for candidate in candidates:
        if candidate.exists() and candidate.stat().st_size > 0:
            return candidate

    for candidate in candidates:
        if candidate.exists():
            return candidate

    raise FileNotFoundError("Could not find results.db in the script directory or its parent.")


def check_column_exists(conn: sqlite3.Connection, table: str, column: str) -> bool:
    """Check if a column already exists in a table."""
    cursor = conn.cursor()
    cursor.execute(f"PRAGMA table_info({table})")
    columns = cursor.fetchall()
    return any(col[1] == column for col in columns)


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Migrate: add problem_class column to runs table"
    )
    parser.add_argument("--db", help="Path to results.db (optional)", default=None)
    args = parser.parse_args()

    db_path = resolve_db_path(args.db)
    print(f"Using database: {db_path}")

    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()

    # Check if column already exists (idempotency)
    if check_column_exists(conn, "runs", "problem_class"):
        print("Column 'problem_class' already exists, skipping ALTER TABLE.")
    else:
        print("Adding 'problem_class' column to runs table...")
        cursor.execute("ALTER TABLE runs ADD COLUMN problem_class TEXT")
        conn.commit()
        print("Column added.")

    # Set any NULL rows to 'provable' (handles partial migration and re-runs)
    cursor.execute("UPDATE runs SET problem_class = 'provable' WHERE problem_class IS NULL")
    rows_updated = cursor.rowcount
    conn.commit()
    if rows_updated:
        print(f"Set {rows_updated} rows to problem_class = 'provable'.")
    else:
        print("No NULL rows to update -- already fully migrated.")

    # Load and display confirmation table
    print("\nConfirmation table (run_id, engine, label, problem_class):")
    print("-" * 80)
    df = pd.read_sql_query(
        "SELECT run_id, engine, label, problem_class FROM runs ORDER BY run_id",
        conn,
    )
    print(df.to_string(index=False))
    print("-" * 80)

    conn.close()
    print("\nMigration complete!")


if __name__ == "__main__":
    main()
