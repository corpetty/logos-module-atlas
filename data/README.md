# data/

| File | What | Kept current? |
|---|---|---|
| `stacks.json` | Module → stack assignment used by the generator | hand-edited |
| `contracts.json` | Result of the last `scripts/fetch_contracts.py` run per module | generated |
| `catalog-pins.tsv` | Catalog submodule → source commit, as of 2026-10-01. The curated `stacks/*.md` were read at these commits | **snapshot** |
| `catalog-index.snapshot.json` | The catalog `index.json` as of 2026-10-01, the same snapshot | **snapshot** |

For current pins use `registry.json` (`.modules[].source`). The snapshots only
record what the hand-written docs were checked against.
