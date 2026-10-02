#!/usr/bin/env python3
"""Report where the hand-written docs have fallen behind the generated registry.

The refresh regenerates registry.json, the cards and the contracts, but not what people
write: stacks/*.md, guides/*.md, gaps.md and data/stacks.json. Each doc names the
Basecamp release it was written against, and the stack docs and guides pin module
versions and commits on a "Pinned at" or "Verified against" line. This compares all of
that with registry.json, and checks that every module has a stack of its own.

Prints a Markdown report. Exit status 0 when nothing has drifted, 1 when something has.

    python3 scripts/check_drift.py

Stdlib only.
"""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# The release a doc was written against, near its top: "Basecamp 0.3.1 (`aeb8192`)",
# "Basecamp `0.3.1` ([aeb8192][bc])" or "Basecamp **0.3.1**"
RELEASE = re.compile(r"Basecamp\s+\**`?(\d+\.\d+\.\d+)`?\**(?:\s*\(\[?`?([0-9a-f]{7,40}))?")
# The paragraph that pins versions
PIN_HEAD = re.compile(r"Pinned at|Verified against", re.I)
# A pinned version, with an optional commit that may name its repo:
# "keystore_module 0.1.0 (`2318c67`)", "liblogos_lez_rln_module 4.2.1 (logos-rln-modules `eb02da8`)"
PIN_VERSION = re.compile(r"\b([a-z][a-z0-9_]*)\s+`?(\d+\.\d+\.\d+)`?(?:\s*\((?:[\w.-]+\s+)?`([0-9a-f]{7,40})`\))?")
# A pinned commit alone: "capability_module `1a1b8b5a`"
PIN_COMMIT = re.compile(r"\b([a-z][a-z0-9_]*)\s+`([0-9a-f]{7,40})`")


def pin_paragraph(text: str) -> str:
    """The lines from the first "Pinned at" or "Verified against" to the next blank line."""
    lines = text.splitlines()
    for i, line in enumerate(lines):
        if PIN_HEAD.search(line):
            out = []
            for following in lines[i:]:
                if not following.strip():
                    break
                out.append(following)
            return " ".join(out)
    return ""


def current_versions(m: dict) -> set[str]:
    """The versions of a module this release ships: the catalog's latest and the bundled one."""
    found = {(m.get("catalog") or {}).get("latest"), (m.get("availability") or {}).get("bundledVersion")}
    return {v for v in found if v}


def check_doc(path: Path, registry: dict, by_name: dict, assigned: dict) -> list[str]:
    text = path.read_text()
    findings = []
    tag = registry["basecamp"]["tag"]
    head = "\n".join(text.splitlines()[:10])

    release = RELEASE.search(head)
    if release and release.group(1) != tag:
        findings.append(f"Written against Basecamp {release.group(1)}; the latest release is {tag}.")
    elif release and release.group(2) and not registry["basecamp"]["commit"].startswith(release.group(2)):
        findings.append(
            f"Pins Basecamp {tag} at `{release.group(2)}`; the release tag points at `{registry['basecamp']['commit'][:7]}`."
        )

    pins = pin_paragraph(text)
    mentioned = set()
    for name, version, commit in PIN_VERSION.findall(pins):
        m = by_name.get(name)
        if not m:
            # A module name: one data/stacks.json knows, or one shaped like one
            if name in assigned or "_" in name:
                findings.append(f"Pins `{name}` {version}, which this release doesn't ship.")
            continue
        mentioned.add(name)
        versions = current_versions(m)
        if versions and version not in versions:
            findings.append(f"Pins `{name}` {version}; the release ships {', '.join(sorted(versions))}.")
        elif commit and not (m.get("source") or {}).get("commit", "").startswith(commit):
            findings.append(f"Pins `{name}` at `{commit}`; the release builds it from `{m['source']['commit'][:7]}`.")
    for name, commit in PIN_COMMIT.findall(pins):
        m = by_name.get(name)
        if not m:
            continue
        mentioned.add(name)
        if not (m.get("source") or {}).get("commit", "").startswith(commit):
            findings.append(f"Pins `{name}` at `{commit}`; the release builds it from `{m['source']['commit'][:7]}`.")

    # A stack doc's pin line names every module in its stack
    stack = path.stem if path.parent.name == "stacks" else None
    if stack in registry["stacks"] and pins:
        for m in registry["modules"]:
            if m["stack"] == stack and m["name"] not in mentioned:
                findings.append(f"`{m['name']}` is in this stack, but the doc doesn't pin it. It may be new: read its card and add it.")
    return findings


def check_stacks(registry: dict, assigned: dict) -> list[str]:
    findings = []
    for m in registry["modules"]:
        if m["name"] not in assigned:
            where = f"in `{m['stack']}` by its category" if m["stack"] in registry["stacks"] else "in no stack"
            findings.append(
                f"`{m['name']}` isn't assigned in data/stacks.json, so the generator put it {where}. Assign it explicitly."
            )
    for key in registry["stacks"]:
        if not (ROOT / "stacks" / f"{key}.md").exists():
            findings.append(f"The `{key}` stack has no doc. Write stacks/{key}.md.")
    return findings


def main() -> int:
    registry = json.loads((ROOT / "registry.json").read_text())
    assigned = json.loads((ROOT / "data/stacks.json").read_text())["modules"]
    by_name = {m["name"]: m for m in registry["modules"]}

    sections: list[tuple[str, list[str]]] = []
    docs = sorted((ROOT / "stacks").glob("*.md")) + sorted((ROOT / "guides").glob("*.md")) + [ROOT / "gaps.md"]
    for path in docs:
        if path.exists():
            found = check_doc(path, registry, by_name, assigned)
            if found:
                sections.append((str(path.relative_to(ROOT)), found))
    found = check_stacks(registry, assigned)
    if found:
        sections.append(("data/stacks.json", found))

    rel = registry["basecamp"]
    # No dates here: the report becomes an issue body, which should change only when the drift does
    print(f"Checked the hand-written docs against `registry.json` (Basecamp {rel['tag']}).")
    print()
    if not sections:
        print("Nothing has drifted.")
        return 0
    for name, items in sections:
        print(f"### {name}")
        print()
        for item in items:
            print(f"- {item}")
        print()
    print(
        "To fix one, check what changed upstream at the commits on the module's card, update the doc "
        "and its pin line, and commit. The refresh closes this issue once nothing has drifted."
    )
    return 1


if __name__ == "__main__":
    sys.exit(main())
