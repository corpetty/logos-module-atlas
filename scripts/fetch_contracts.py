#!/usr/bin/env python3
"""Build each module's canonical LIDL contract from its pinned source commit.

Module flakes built with logos-module-builder expose `packages.<system>.lidl` —
the `.lidl` contract every SDK's codegen consumes (C++, Rust, Nim). This builds
it for every module in registry.json, at the exact commit the registry pins,
and stores it as modules/<name>/<name>.lidl with the commit in
modules/<name>/.contract-commit. A contract whose stamp already matches is
skipped, so reruns only rebuild what moved.

Needs `nix` with flakes. Builds are mostly substituted from the Logos cache
(https://cache.nix.logos.co/public) — ~25 s each when cached.

    python3 scripts/fetch_contracts.py              # all modules, 4 at a time
    python3 scripts/fetch_contracts.py keystore_module lez_core
    python3 scripts/fetch_contracts.py --force -j 2

Results (including why a module has no contract) go to data/contracts.json.
Rerun scripts/build_registry.py afterwards so the cards pick the contracts up.
"""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def flake_ref(src: dict) -> str:
    ref = f"github:{src['repo']}/{src['commit']}"
    return ref + (f"?dir={src['subdir']}" if src.get("subdir") else "")


def build_one(mod: dict, force: bool, timeout: int) -> dict:
    name, src = mod["name"], mod.get("source")
    dest = ROOT / "modules" / name
    stamp = dest / ".contract-commit"
    if not src:
        return {"name": name, "status": "no-source"}
    ref = f"{flake_ref(src)}#lidl"
    # Same record whether built now or earlier, so an unchanged run leaves data/contracts.json alone.
    ok = {"name": name, "status": "ok", "commit": src["commit"], "ref": ref}
    if not force and stamp.exists() and stamp.read_text().strip() == src["commit"] and (dest / f"{name}.lidl").exists():
        return {**ok, "_action": "cached"}
    try:
        p = subprocess.run(
            ["nix", "build", ref, "--no-link", "--print-out-paths"],
            capture_output=True, text=True, timeout=timeout,
        )
    except subprocess.TimeoutExpired:
        return {"name": name, "status": "timeout", "ref": ref}
    if p.returncode != 0:
        err = p.stderr.strip().splitlines()
        reason = next((l.strip() for l in err if "does not provide attribute" in l), None) or (err[-1].strip() if err else "unknown")
        return {"name": name, "status": "failed", "ref": ref, "reason": reason[:300]}
    out = Path(p.stdout.strip().splitlines()[-1])
    files = sorted(out.rglob("*.lidl")) if out.is_dir() else [out]
    pick = next((f for f in files if f.stem == name), files[0] if files else None)
    if not pick:
        return {"name": name, "status": "empty", "ref": ref}
    dest.mkdir(parents=True, exist_ok=True)
    target = dest / f"{name}.lidl"
    shutil.copyfile(pick, target)
    target.chmod(0o644)
    stamp.write_text(src["commit"] + "\n")
    return {**ok, "_action": "built"}


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("modules", nargs="*", help="only these modules (default: every module in registry.json)")
    ap.add_argument("-j", "--jobs", type=int, default=4)
    ap.add_argument("--force", action="store_true", help="rebuild even when the stamp matches")
    ap.add_argument("--timeout", type=int, default=1200, help="seconds per build")
    ap.add_argument("--include-ui", action="store_true", help="also try ui_qml modules (most have no #lidl)")
    args = ap.parse_args()

    registry = json.loads((ROOT / "registry.json").read_text())
    mods = [
        m for m in registry["modules"]
        if (not args.modules or m["name"] in args.modules) and (args.include_ui or m["type"] != "ui_qml")
    ]
    results_path = ROOT / "data" / "contracts.json"
    previous = {r["name"]: r for r in json.loads(results_path.read_text())} if results_path.exists() else {}

    with ThreadPoolExecutor(args.jobs) as pool:
        for r in pool.map(lambda m: build_one(m, args.force, args.timeout), mods):
            action = r.pop("_action", r["status"])
            previous[r["name"]] = r
            print(f"{action:>8}  {r['name']}" + (f"  — {r['reason']}" if r.get("reason") else ""), file=sys.stderr, flush=True)

    results_path.write_text(json.dumps(sorted(previous.values(), key=lambda r: r["name"]), indent=2) + "\n")


if __name__ == "__main__":
    main()
