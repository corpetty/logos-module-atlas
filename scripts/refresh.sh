#!/usr/bin/env bash
# Regenerate everything: resolve the release + catalog, build contracts that moved,
# then re-render so the cards pick the contracts up.
set -euo pipefail
cd "$(dirname "$0")/.."
python3 scripts/build_registry.py "$@"
python3 scripts/fetch_contracts.py -j "${JOBS:-4}"
python3 scripts/build_registry.py "$@"
