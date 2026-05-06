import sqlite3
from pathlib import Path

def resolve_db_path() -> Path:
  base_dir = Path(__file__).resolve().parent
  candidates = [base_dir / "results.db", base_dir.parent / "results.db"]

  for candidate in candidates:
    if candidate.exists() and candidate.stat().st_size > 0:
      return candidate

  for candidate in candidates:
    if candidate.exists():
      return candidate

  raise FileNotFoundError("Could not find results.db in the script directory or its parent.")


data_path = resolve_db_path()
conn = sqlite3.connect(data_path)
cur = conn.cursor()

print(f"Using database: {data_path}")
print("Tables:")
tables = cur.execute("""
  SELECT name
  FROM sqlite_master
  WHERE type='table'
  ORDER BY name
""").fetchall()

for (table_name,) in tables:
  print(f"\n[{table_name}]")
  cols = cur.execute(f"PRAGMA table_info({table_name})").fetchall()
  for col in cols:
    print(col)
