import sqlite3, pandas as pd, pathlib, re, os

os.chdir(pathlib.Path(__file__).parent.parent)  # run from repo root

conn_prov = sqlite3.connect('analysis/tptp-provable-results.db')
runs_p = pd.read_sql_query('SELECT * FROM runs', conn_prov)
res_p = pd.read_sql_query('SELECT * FROM results', conn_prov)
df_p = res_p.merge(runs_p, on='run_id')

def get_rating(raw_path):
    # paths are stored relative to analysis/ dir: ..\TPTP-v9.2.1\...
    # strip leading dots/slashes to get path relative to repo root
    parts = pathlib.PureWindowsPath(raw_path).parts
    # drop the leading '..' component
    clean_parts = [p for p in parts if p not in ('..', '.')]
    p = pathlib.Path(*clean_parts)
    if not p.exists():
        return None
    try:
        with open(p, encoding='utf-8', errors='ignore') as f:
            for line in f:
                m = re.search(r'Rating\s*:\s*([0-9.]+)', line)
                if m:
                    return float(m.group(1))
                if line.strip() and not line.startswith('%'):
                    break
    except Exception:
        return None
    return None

pid = df_p[df_p['engine'] == 'priority-id'].copy()
pid['rating'] = pid['path'].apply(get_rating)

print('Rating coverage:', pid['rating'].notna().sum(), 'of', len(pid))
solved = pid[pid['status'] == 'provable']
unsolved = pid[pid['status'] != 'provable']

print()
print('Solved rating stats:')
print(solved['rating'].describe().round(3))
print()
print('Unsolved rating stats:')
print(unsolved['rating'].describe().round(3))
print()
print('Max rating solved:', solved['rating'].max())
print()
bins = [0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.01]
print('Solved rating distribution:')
print(pd.cut(solved['rating'].dropna(), bins=bins).value_counts().sort_index())
print()
print('Unsolved rating distribution:')
print(pd.cut(unsolved['rating'].dropna(), bins=bins).value_counts().sort_index())

# Also check: what fraction of problems with rating <= 0.2 are solved?
low = pid[pid['rating'] <= 0.2]
print()
print(f'Problems with rating <= 0.2: {len(low)}, solved: {(low["status"]=="provable").sum()}')
med = pid[(pid['rating'] > 0.2) & (pid['rating'] <= 0.5)]
print(f'Problems with 0.2 < rating <= 0.5: {len(med)}, solved: {(med["status"]=="provable").sum()}')
high = pid[pid['rating'] > 0.5]
print(f'Problems with rating > 0.5: {len(high)}, solved: {(high["status"]=="provable").sum()}')
