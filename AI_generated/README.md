# AI-Generated FOF Benchmark Problems

Generated TPTP FOF problems for benchmarking the theorem prover.

## Files

- `easy.p` - 15 propositional logic problems (5-20 atoms)
- `medium.p` - 30 first-order problems with universal quantifiers (20-50 atoms)
- `hard.p` - 30 problems with nested quantifiers (50-100 atoms)
- `expert.p` - 25 deeply nested complex problems (100-150 atoms)

## Statistics

- Total problems: 100+
- Provable: 50/50 unprovable split
- Format: TPTP FOF (.p files)

## Generation

Run the generator:
```bash
python scripts/generate_fof_benchmarks.py
```

Verify with tests:
```bash
python tests/test_fof_generation.py
```
