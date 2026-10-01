#!/usr/bin/env python3
"""Build the Logos Module Atlas from what the latest Basecamp release makes available.

Scope is resolved, never hard-coded:

  * the latest `logos-co/logos-basecamp` GitHub release (or --basecamp-tag);
  * the modules that release BUNDLES — the `installedDistributed` list in its
    flake.nix, each pinned by the release's flake.lock;
  * every package in the catalog that release's package_downloader talks to by
    default (`kDefaultRepositoryUrl`, compiled in), plus any catalogs it includes.

Each catalog package is traced to the exact source commit it was built from the
same way logos-release-set's resolver does it:

  publisherRef tag -> catalog commit -> submodule gitlink -> metadata.json name

Outputs (regenerated every run — do not hand-edit):
  registry.json                      machine-readable registry
  llms.txt                           AI entry point (llmstxt.org format)
  modules/<name>/README.md           per-module card
  modules/<name>/interface.<ext>     the annotated interface source, extracted
  README.md                          only the block between the registry markers

Curated inputs (never written here): modules/<name>/NOTES.md, stacks/, guides/,
data/stacks.json. Canonical contracts modules/<name>/<name>.lidl come from
scripts/fetch_contracts.py and are read here if present.

Stdlib only. Auth: GITHUB_TOKEN / GH_TOKEN, else `gh auth token`.
"""

from __future__ import annotations

import argparse
import datetime as dt
import json
import os
import re
import subprocess
import sys
import urllib.error
import urllib.request
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BASECAMP_REPO = "logos-co/logos-basecamp"
DOWNLOADER_REPO = "logos-co/logos-package-downloader"
FALLBACK_DEFAULT_REPO_URL = (
    "https://raw.githubusercontent.com/logos-co/logos-modules-release/refs/heads/main/logos-repo.json"
)
ATLAS_REPO = "corpetty/logos-module-atlas"
RAW_BASE = f"https://raw.githubusercontent.com/{ATLAS_REPO}/main"

# Paths inside a source repo that never hold the module's own metadata/interface.
NOISE = re.compile(
    r"(^|/)(doctests?|tests?|test|examples?|fixtures|probes?|vendor|third_party|deps|"
    r"node_modules|generated_code|build|result|\.github)(/|$)|-probe(/|$)"
)
DOC_EXT = (".md", ".rst")

# --------------------------------------------------------------------------- http


def _token() -> str | None:
    for var in ("GITHUB_TOKEN", "GH_TOKEN"):
        if os.environ.get(var):
            return os.environ[var]
    try:
        return subprocess.run(
            ["gh", "auth", "token"], capture_output=True, text=True, check=True
        ).stdout.strip() or None
    except (OSError, subprocess.CalledProcessError):
        return None


TOKEN = _token()
_cache: dict[str, object] = {}


def _get(url: str, accept: str = "application/vnd.github+json") -> bytes | None:
    headers = {"Accept": accept, "User-Agent": "logos-module-atlas"}
    if TOKEN and url.startswith("https://api.github.com/"):
        headers["Authorization"] = f"Bearer {TOKEN}"
    for attempt in range(3):
        try:
            with urllib.request.urlopen(urllib.request.Request(url, headers=headers), timeout=60) as r:
                return r.read()
        except urllib.error.HTTPError as e:
            if e.code in (404, 409, 422):  # missing, empty repo, or a sha GitHub cannot serve
                if e.code != 404:
                    print(f"warn: HTTP {e.code} for {url}", file=sys.stderr)
                return None
            if attempt == 2:
                raise
        except urllib.error.URLError:
            if attempt == 2:
                raise
    return None


def api(path: str):
    url = path if path.startswith("http") else f"https://api.github.com{path}"
    if url not in _cache:
        body = _get(url)
        _cache[url] = json.loads(body) if body else None
    return _cache[url]


def raw(repo: str, ref: str, path: str) -> str | None:
    key = f"raw:{repo}@{ref}:{path}"
    if key not in _cache:
        body = _get(f"https://raw.githubusercontent.com/{repo}/{ref}/{path}", accept="*/*")
        _cache[key] = body.decode("utf-8", "replace") if body is not None else None
    return _cache[key]  # type: ignore[return-value]


def fetch_json(url: str):
    body = _get(url, accept="application/json")
    return json.loads(body) if body else None


def tree(repo: str, sha: str) -> list[str]:
    t = api(f"/repos/{repo}/git/trees/{sha}?recursive=1") or {}
    return [e["path"] for e in t.get("tree", []) if e.get("type") == "blob"]


def tag_commit(repo: str, tag: str) -> str | None:
    ref = api(f"/repos/{repo}/git/ref/tags/{tag}")
    if not ref:
        return None
    obj = ref["object"]
    while obj["type"] == "tag":  # annotated tag -> dereference
        obj = api(f"/repos/{repo}/git/tags/{obj['sha']}")["object"]
    return obj["sha"]


def permalink(repo: str, sha: str, path: str = "") -> str:
    kind = "blob" if path and "." in path.rsplit("/", 1)[-1] else "tree"
    return f"https://github.com/{repo}/{kind}/{sha}/{path}".rstrip("/")


def slug_from_url(url: str) -> str:
    m = re.search(r"github(?:usercontent)?\.com/([^/]+)/([^/.]+)", url)
    return f"{m.group(1)}/{m.group(2)}" if m else url


# ------------------------------------------------------------- basecamp release


def basecamp_release(tag: str | None) -> dict:
    rel = api(f"/repos/{BASECAMP_REPO}/releases/tags/{tag}") if tag else api(
        f"/repos/{BASECAMP_REPO}/releases/latest"
    )
    if not rel:
        sys.exit(f"cannot find Basecamp release {tag or 'latest'}")
    tag = rel["tag_name"]
    return {
        "tag": tag,
        "name": rel.get("name") or tag,
        "publishedAt": rel.get("published_at"),
        "url": rel.get("html_url"),
        "commit": tag_commit(BASECAMP_REPO, tag),
    }


def bundled_inputs(flake_nix: str) -> list[str]:
    """installedDistributed's entries, each resolved through let-bindings to a flake input."""
    m = re.search(r"installedDistributed\s*=\s*map\s+\w+\s*\[(.*?)\];", flake_nix, re.S)
    if not m:
        return []
    inputs = set(re.findall(r"^\s*([\w-]+)\.url\s*=", flake_nix, re.M))
    out = []
    for var in re.findall(r"^\s*([\w-]+)\s*$", re.sub(r"#.*", "", m.group(1)), re.M):
        name = var
        for _ in range(6):
            if name in inputs:
                break
            d = re.search(rf"^\s*{re.escape(name)}\s*=\s*([\w-]+)", flake_nix, re.M)
            if not d:
                break
            name = d.group(1)
        if name in inputs:
            out.append(name)
    return out


def locked(lock: dict, node_name: str) -> dict | None:
    node = lock["nodes"].get(lock["nodes"]["root"]["inputs"].get(node_name, node_name))
    return node.get("locked") if node else None


def default_repo_url(lock: dict) -> tuple[str, str]:
    """kDefaultRepositoryUrl as compiled into the release's package downloader."""
    for node in lock["nodes"].values():
        lk = node.get("locked") or {}
        if f"{lk.get('owner')}/{lk.get('repo')}" == DOWNLOADER_REPO:
            src = raw(DOWNLOADER_REPO, lk["rev"], "src/package_downloader_lib.cpp") or ""
            m = re.search(r'kDefaultRepositoryUrl\s*=\s*"([^"]+)"', src)
            if m:
                return m.group(1), f"{DOWNLOADER_REPO}@{lk['rev']}"
    return FALLBACK_DEFAULT_REPO_URL, "fallback constant (not found in lock)"


# ------------------------------------------------------------------ source repos


def module_metadata(repo: str, sha: str) -> list[tuple[str, dict]]:
    """Every (subdir, metadata.json) in a repo that is a real module, not a test probe."""
    found = []
    for path in tree(repo, sha):
        if not path.endswith("metadata.json") or path.count("/") > 1 or NOISE.search(path):
            continue
        text = raw(repo, sha, path)
        try:
            meta = json.loads(text or "")
        except json.JSONDecodeError:
            continue
        if isinstance(meta, dict) and meta.get("name"):
            found.append((path.rsplit("/", 1)[0] if "/" in path else "", meta))
    return found


def parse_gitmodules(text: str) -> dict[str, str]:
    urls, path = {}, None
    for line in text.splitlines():
        line = line.strip()
        if line.startswith("path"):
            path = line.split("=", 1)[1].strip()
        elif line.startswith("url") and path:
            urls[path] = line.split("=", 1)[1].strip()
    return urls


def catalog_module_map(catalog_repo: str, commit: str) -> dict[str, dict]:
    """module name -> {dir, repo, sha, subdir} for every submodule at a catalog commit."""
    key = f"map:{catalog_repo}@{commit}"
    if key in _cache:
        return _cache[key]  # type: ignore[return-value]
    urls = parse_gitmodules(raw(catalog_repo, commit, ".gitmodules") or "")
    listing = api(f"/repos/{catalog_repo}/contents/submodules?ref={commit}") or []
    subs = [
        (e["name"], e["sha"], slug_from_url(urls.get(f"submodules/{e['name']}", "")))
        for e in listing
        if e["name"] != ".gitkeep" and f"submodules/{e['name']}" in urls
    ]
    out: dict[str, dict] = {}
    with ThreadPoolExecutor(8) as pool:
        for (d, sha, repo), metas in zip(subs, pool.map(lambda s: module_metadata(s[2], s[1]), subs)):
            for subdir, meta in metas:
                out.setdefault(meta["name"], {"dir": d, "repo": repo, "sha": sha, "subdir": subdir})
    _cache[key] = out
    return out


# --------------------------------------------------------------- interface files


def find_interface(repo: str, sha: str, subdir: str, meta: dict, paths: list[str]) -> dict:
    """Locate the files that define a module's API, in order of authority."""
    pre = f"{subdir}/" if subdir else ""
    own = [p for p in paths if p.startswith(pre) and not NOISE.search(p[len(pre):])]
    name = meta["name"]
    found: dict = {"lidl": [], "rust": None, "cpp": None, "rep": None, "qml": None, "docs": []}

    found["lidl"] = [p for p in own if p.endswith(".lidl")]
    cg = meta.get("codegen") or {}
    rust = cg.get("rust") or {}
    if rust.get("source"):
        p = f"{pre}{rust.get('crate', '')}/{rust['source']}".replace("//", "/")
        if p in paths:
            found["rust"] = {"path": p, "trait": rust.get("trait")}
    for cand in (cg.get("impl_header"), f"src/{name}_impl.h"):
        if cand and f"{pre}{cand}" in paths:
            found["cpp"] = f"{pre}{cand}"
            break
    if not found["cpp"]:
        # Impl headers not named after the module (monero_wallet_core_impl.h), then legacy plugins.
        for pat in (r"src/[^/]*_impl\.h$", r"src/[^/]*(_plugin|_interface)\.h$"):
            hits = sorted((p for p in own if re.search(pat, p)), key=len)
            if hits:
                found["cpp"] = hits[0]
                break
    rep = cg.get("rep")
    reps = [f"{pre}{rep}"] if rep and f"{pre}{rep}" in paths else [p for p in own if p.endswith(".rep")]
    found["rep"] = reps[0] if reps else None
    if meta.get("view") and f"{pre}{meta['view']}" in paths:
        found["qml"] = f"{pre}{meta['view']}"

    for p in own:
        rel = p[len(pre):]
        top = rel.split("/")[0]
        if rel in ("README.md", "CLAUDE.md", "AGENTS.md", "SPEC.md") or (
            top == "docs" and rel.endswith(DOC_EXT) and "/_" not in rel
        ):
            found["docs"].append(p)
    found["docs"].sort(key=lambda p: (p.count("/"), p))
    return found


def rust_trait_block(src: str, trait: str | None) -> str | None:
    m = re.search(rf"((?:^[ \t]*///.*\n)*)^[ \t]*pub trait {re.escape(trait or '')}\w*\b[^{{]*\{{", src, re.M)
    if not m:
        return None
    depth, i = 0, m.end() - 1
    while i < len(src):
        depth += {"{": 1, "}": -1}.get(src[i], 0)
        if depth == 0:
            return src[m.start(): i + 1] + "\n"
        i += 1
    return None


def write_interface(dest: Path, repo: str, sha: str, iface: dict, name: str) -> dict | None:
    """Copy the most authoritative annotated interface source next to the card."""
    for f in dest.glob("interface.*"):
        f.unlink()
    choice = None
    own_lidl = [p for p in iface["lidl"] if Path(p).stem in (name, f"lib{name}") or Path(p).stem.endswith(name)]
    if own_lidl:
        choice = (own_lidl[0], "lidl", raw(repo, sha, own_lidl[0]))
    elif iface["rust"]:
        src = raw(repo, sha, iface["rust"]["path"]) or ""
        block = rust_trait_block(src, iface["rust"]["trait"])
        if block:
            choice = (iface["rust"]["path"], "rs", block)
    if not choice and iface["cpp"]:
        choice = (iface["cpp"], "h", raw(repo, sha, iface["cpp"]))
    if not choice and iface["rep"]:
        choice = (iface["rep"], "rep", raw(repo, sha, iface["rep"]))
    if not choice or not choice[2]:
        return None
    path, ext, text = choice
    header = {
        "lidl": ";", "rs": "//", "h": "//", "rep": "//",
    }[ext]
    banner = (
        f"{header} Extracted by logos-module-atlas from {repo}@{sha[:12]}:{path}\n"
        f"{header} {permalink(repo, sha, path)}\n\n"
    )
    (dest / f"interface.{ext}").write_text(banner + text)
    return {"file": f"interface.{ext}", "from": path, "url": permalink(repo, sha, path), "lang": ext}


# ------------------------------------------------------------------ API summary

LIDL_DECL = re.compile(
    r'^\s*(method|event)\s+(\w+)\s*\(([^)]*)\)(?:\s*->\s*([\w\[\]<>?]+))?(?:\s+description\s+"((?:[^"\\]|\\.)*)")?',
    re.M,
)


def first_sentence(text: str) -> str:
    text = re.sub(r"\s+", " ", text.replace("\\n", " ").replace('\\"', '"')).strip()
    s = text
    for m in re.finditer(r"[.!?](?=\s|$)", text):
        head = text[: m.end()]
        # Not inside `code`, { json }, ( parens ) or [ lists ].
        if head.count("`") % 2 == 0 and head.count("{") <= head.count("}") \
                and head.count("(") <= head.count(")") and head.count("[") <= head.count("]"):
            s = head
            break
    return s if len(s) <= 220 else s[:217] + "…"


def lidl_api(text: str) -> dict:
    out: dict = {"methods": [], "events": []}
    pending: list[str] = []
    for line in text.splitlines():
        s = line.strip()
        if s.startswith(";"):
            pending.append(s.lstrip("; "))
            continue
        m = LIDL_DECL.match(line)
        if m:
            kind, n, args, ret, desc = m.groups()
            entry = {
                "name": n,
                "signature": f"{n}({args.strip()})" + (f" -> {ret}" if ret else ""),
                "summary": first_sentence(desc or " ".join(pending)),
            }
            out["methods" if kind == "method" else "events"].append(entry)
        if s:
            pending = []
    return out


def rep_api(text: str) -> dict:
    """Qt Remote Objects view contract: SLOTs are callable, PROPs are bound state, SIGNALs push."""
    out: dict = {"methods": [], "events": []}
    for kind, body in re.findall(r"\b(SLOT|PROP|SIGNAL)\s*\((.*)\)\s*;?\s*$", text, re.M):
        body = body.strip()
        if kind == "SLOT":
            m = re.match(r"(?:([\w:<>]+)\s+)?(\w+)\s*\((.*)\)$", body)
            if m:
                ret, n, args = m.groups()
                out["methods"].append({"name": n, "signature": f"{n}({args})" + (f" -> {ret}" if ret and ret != "void" else ""), "summary": "slot"})
        elif kind == "PROP":
            m = re.match(r"([\w:<>]+)\s+(\w+)", body)
            if m:
                out["methods"].append({"name": m.group(2), "signature": f"{m.group(2)}: {m.group(1)}", "summary": "property" + (" (read-only)" if "READONLY" in body else "")})
        else:
            m = re.match(r"(\w+)\s*\((.*)\)$", body)
            if m:
                out["events"].append({"name": m.group(1), "signature": f"{m.group(1)}({m.group(2)})", "summary": "signal"})
    return out


def rust_api(block: str) -> dict:
    out: dict = {"methods": [], "events": []}
    docs: list[str] = []
    for line in block.splitlines():
        s = line.strip()
        if s.startswith("///"):
            docs.append(s[3:].strip())
            continue
        m = re.match(r"fn (\w+)\(\s*&(?:mut )?self\s*,?\s*([^)]*)\)\s*(?:->\s*([^;{]+))?", s)
        if m:
            n, args, ret = m.groups()
            out["methods"].append({
                "name": n,
                "signature": f"{n}({args.strip()})" + (f" -> {ret.strip()}" if ret else ""),
                "summary": first_sentence(" ".join(docs)),
            })
        if s and not s.startswith("#["):
            docs = []
    return out


# ------------------------------------------------------------------- the build


def resolve_catalog(repo_url: str) -> tuple[list[dict], list[dict]]:
    """Every catalog reachable from the default repo URL (includes followed), with its index."""
    catalogs, packages, seen, queue = [], [], set(), [repo_url]
    while queue:
        url = queue.pop(0)
        if url in seen:
            continue
        seen.add(url)
        card = fetch_json(url)
        if not card:
            print(f"warn: cannot fetch catalog {url}", file=sys.stderr)
            continue
        index = fetch_json(card["indexUrl"]) or {"packages": []}
        catalog_repo = slug_from_url(url)
        catalogs.append({
            "url": url,
            "repo": catalog_repo,
            "name": card.get("name"),
            "displayName": card.get("displayName"),
            "indexUrl": card["indexUrl"],
            "indexGeneratedAt": index.get("generatedAt"),
            "packageCount": len(index.get("packages", [])),
        })
        for inc in (card.get("includesUrl") or card.get("includes") or []):
            queue.append(inc if isinstance(inc, str) else inc.get("url"))
        for pkg in index.get("packages", []):
            packages.append({"catalog": catalog_repo, "catalogName": card.get("displayName"), **pkg})
    return catalogs, packages


def semver_key(v: str):
    core, _, pre = v.partition("-")
    nums = [int(x) if x.isdigit() else 0 for x in core.split(".")]
    return (nums, pre == "", pre)


def dep_names(deps) -> list[dict]:
    out = []
    for d in deps or []:
        out.append({"name": d, "version": None} if isinstance(d, str) else {"name": d["name"], "version": d.get("version")})
    return out


def build(args) -> None:
    release = basecamp_release(args.basecamp_tag)
    print(f"Basecamp {release['tag']} @ {release['commit'][:12]}", file=sys.stderr)
    flake_nix = raw(BASECAMP_REPO, release["commit"], "flake.nix") or ""
    lock = json.loads(raw(BASECAMP_REPO, release["commit"], "flake.lock") or "{}")
    repo_url, repo_url_source = default_repo_url(lock)
    catalogs, packages = resolve_catalog(repo_url)
    stacks = json.loads((ROOT / "data/stacks.json").read_text())

    entries: dict[str, dict] = {}

    # Bundled modules: pinned by the release's own flake.lock.
    for inp in bundled_inputs(flake_nix):
        lk = locked(lock, inp)
        if not lk or lk.get("type") != "github":
            continue
        repo, sha = f"{lk['owner']}/{lk['repo']}", lk["rev"]
        for subdir, meta in module_metadata(repo, sha):
            entries[meta["name"]] = {
                "name": meta["name"],
                "meta": meta,
                "source": {"repo": repo, "commit": sha, "subdir": subdir, "pinnedBy": f"{BASECAMP_REPO}@{release['tag']} flake.lock ({inp})"},
                "bundled": {"version": meta.get("version"), "flakeInput": inp},
                "catalog": None,
            }

    # Catalog packages: traced to the commit their latest release was built from.
    def trace(pkg: dict) -> dict:
        versions = sorted(pkg["versions"], key=lambda v: semver_key(v["manifest"]["version"]), reverse=True)
        latest = versions[0]
        commit = tag_commit(pkg["catalog"], latest["publisherRef"]) if latest.get("publisherRef") else None
        mapping = catalog_module_map(pkg["catalog"], commit) if commit else {}
        src = mapping.get(pkg["name"])
        return {"pkg": pkg, "versions": versions, "latest": latest, "catalogCommit": commit, "src": src}

    with ThreadPoolExecutor(6) as pool:
        traced = list(pool.map(trace, packages))

    for t in traced:
        pkg, latest, src = t["pkg"], t["latest"], t["src"]
        man = latest["manifest"]
        catalog = {
            "repository": pkg["catalogName"],
            "catalogRepo": pkg["catalog"],
            "latest": man["version"],
            "versions": [v["manifest"]["version"] for v in t["versions"]],
            "releasedAt": latest.get("releasedAt"),
            "publisherRef": latest.get("publisherRef"),
            "catalogCommit": t["catalogCommit"],
            "lgx": latest.get("url"),
            "sha256": latest.get("sha256"),
            "size": latest.get("size"),
            "variants": sorted(k.split("/", 1)[1] for k in (man.get("hashes") or {}) if k.startswith("variants/")),
        }
        e = entries.setdefault(pkg["name"], {"name": pkg["name"], "bundled": None})
        e["catalog"] = catalog
        e["manifest"] = man
        if src:
            meta = next((m for sd, m in module_metadata(src["repo"], src["sha"]) if m["name"] == pkg["name"]), {})
            e.setdefault("meta", meta)
            e["source"] = {
                "repo": src["repo"], "commit": src["sha"], "subdir": src["subdir"],
                "pinnedBy": f"{pkg['catalog']}@{(t['catalogCommit'] or '')[:12]} submodules/{src['dir']} (tag {latest.get('publisherRef')})",
            }
        else:
            e.setdefault("meta", {})
            e.setdefault("source", None)
            print(f"warn: no source traced for {pkg['name']} {man['version']}", file=sys.stderr)

    _KNOWN.clear()
    _KNOWN.update(entries)

    # Dependency graph.
    for e in entries.values():
        man = e.get("manifest") or e["meta"]
        e["dependencies"] = dep_names(man.get("dependencies"))
    for e in entries.values():
        e["requiredBy"] = sorted(o["name"] for o in entries.values() if any(d["name"] == e["name"] for d in o["dependencies"]))

    modules_dir = ROOT / "modules"
    modules_dir.mkdir(exist_ok=True)
    for stale in modules_dir.iterdir():
        if stale.is_dir() and stale.name not in entries and not (stale / "NOTES.md").exists():
            for f in stale.iterdir():
                f.unlink()
            stale.rmdir()

    def render_one(e: dict) -> dict:
        name, meta, src = e["name"], e.get("meta") or {}, e.get("source")
        dest = modules_dir / name
        dest.mkdir(exist_ok=True)
        rec = registry_record(e, release, stacks)
        if src:
            paths = tree(src["repo"], src["commit"])
            iface = find_interface(src["repo"], src["commit"], src["subdir"], meta, paths)
            rec["interfaceSource"] = write_interface(dest, src["repo"], src["commit"], iface, name)
            rec["docs"] = [{"path": p, "url": permalink(src["repo"], src["commit"], p)} for p in iface["docs"]]
            rec["qmlView"] = iface["qml"]
        contract = dest / f"{name}.lidl"
        stamp = dest / ".contract-commit"
        if contract.exists():
            rec["contract"] = {
                "file": contract.name,
                "builtFrom": stamp.read_text().strip() if stamp.exists() else None,
                "stale": bool(src and stamp.exists() and stamp.read_text().strip() != src["commit"]),
            }
        api_text = contract.read_text() if contract.exists() else None
        if api_text:
            rec["api"] = lidl_api(api_text)
        elif rec.get("interfaceSource"):
            text = (dest / rec["interfaceSource"]["file"]).read_text()
            parser = {"lidl": lidl_api, "rs": rust_api, "rep": rep_api}.get(rec["interfaceSource"]["lang"])
            rec["api"] = parser(text) if parser else None
        rec["notes"] = (dest / "NOTES.md").exists()
        (dest / "README.md").write_text(render_card(rec, release))
        return rec

    with ThreadPoolExecutor(6) as pool:
        records = sorted(pool.map(render_one, entries.values()), key=lambda r: (r["stack"], r["name"]))

    registry = {
        "schemaVersion": 1,
        "generatedAt": dt.datetime.now(dt.timezone.utc).replace(microsecond=0).isoformat(),
        "scope": "Every module the latest Logos Basecamp release makes available: bundled in the app, or installable from its default catalog.",
        "basecamp": release,
        "defaultRepositoryUrl": repo_url,
        "defaultRepositoryUrlSource": repo_url_source,
        "catalogs": catalogs,
        "stacks": stacks["stacks"],
        "modules": records,
    }
    (ROOT / "registry.json").write_text(json.dumps(registry, indent=2) + "\n")
    (ROOT / "llms.txt").write_text(render_llms(registry))
    splice_readme(registry)
    print(f"{len(records)} modules -> registry.json", file=sys.stderr)


def registry_record(e: dict, release: dict, stacks: dict) -> dict:
    meta, man, cat = e.get("meta") or {}, e.get("manifest") or {}, e.get("catalog")
    name = e["name"]
    stack = stacks["modules"].get(name) or stacks["byCategory"].get(man.get("category") or meta.get("category") or "", "other")
    src = e.get("source")
    return {
        "name": name,
        "displayName": man.get("display_name") or meta.get("display_name"),
        "description": man.get("description") or meta.get("description"),
        "type": man.get("type") or meta.get("type"),
        "interface": meta.get("interface"),
        "language": language_of(meta),
        "category": man.get("category") or meta.get("category"),
        "stack": stack,
        "availability": {
            "bundledInBasecamp": bool(e.get("bundled")),
            "bundledVersion": (e.get("bundled") or {}).get("version"),
            "inDefaultCatalog": bool(cat),
        },
        "catalog": cat,
        "source": src and {**src, "url": permalink(src["repo"], src["commit"], src["subdir"])},
        "dependencies": e["dependencies"],
        "requiredBy": e["requiredBy"],
        "concurrency": meta.get("concurrency"),
        "capabilities": meta.get("capabilities") or [],
        "provides": meta.get("provides") or [],
        "uses": meta.get("uses") or [],
        "card": f"modules/{name}/README.md",
    }


def language_of(meta: dict) -> str | None:
    cg = meta.get("codegen") or {}
    if "rust" in cg:
        return "rust"
    if "nim" in cg or meta.get("interface") == "nim":
        return "nim"
    if meta.get("type") == "ui_qml":
        return "qml" + ("+c++" if cg.get("rep") or meta.get("interface") == "universal" else "")
    if meta.get("interface") in ("universal", "legacy", "cdylib") or cg.get("impl_header"):
        return "c++"
    return None


# ------------------------------------------------------------------- rendering


def md_link_mod(name: str, known: set[str], prefix: str = "../") -> str:
    return f"[`{name}`]({prefix}{name}/README.md)" if name in known else f"`{name}`"


_KNOWN: set[str] = set()


def render_card(r: dict, release: dict) -> str:
    known = _KNOWN
    src, cat, av = r.get("source"), r.get("catalog"), r["availability"]
    where = []
    if av["bundledInBasecamp"]:
        where.append(f"bundled in Basecamp {release['tag']} (v{av['bundledVersion']})")
    if av["inDefaultCatalog"]:
        where.append(f"installable from the default catalog *{cat['repository']}*")
    deps = ", ".join(
        md_link_mod(d["name"], known) + (f" `{d['version']}`" if d.get("version") else "") for d in r["dependencies"]
    ) or "—"
    req = ", ".join(md_link_mod(n, known) for n in r["requiredBy"]) or "—"
    L = [
        f"<!-- GENERATED by scripts/build_registry.py — do not edit. Curated notes go in NOTES.md. -->",
        f"# {r['name']}" + (f" — {r['displayName']}" if r.get("displayName") and r["displayName"] != r["name"] else ""),
        "",
        f"> {r['description'] or '(no description in metadata)'}",
        "",
        "| | |",
        "|---|---|",
        f"| Type | `{r['type']}`" + (f" · interface `{r['interface']}`" if r.get("interface") else "") + (f" · {r['language']}" if r.get("language") else "") + " |",
        f"| Stack | [{r['stack']}](../../stacks/{r['stack']}.md) · category `{r['category']}` |",
        f"| Availability | {'; '.join(where)} |",
    ]
    if cat:
        L.append(f"| Catalog version | **{cat['latest']}** (released {(cat['releasedAt'] or '')[:10]}) · all: {', '.join(cat['versions'])} |")
        L.append(f"| Platforms | {', '.join(cat['variants']) or 'unknown'} |")
        L.append(f"| Package | [{cat['publisherRef']}]({cat['lgx']}) · sha256 `{(cat['sha256'] or '')[:16]}…` |")
    if src:
        sub = f" (`{src['subdir']}/`)" if src.get("subdir") else ""
        L.append(f"| Source | [{src['repo']}@{src['commit'][:7]}]({src['url']}){sub} |")
        L.append(f"| Pinned by | {src['pinnedBy']} |")
    L.append(f"| Depends on | {deps} |")
    L.append(f"| Required by | {req} |")
    if r.get("concurrency"):
        L.append(f"| Concurrency | `{r['concurrency']}` |")
    L += ["", "## Read these first", ""]
    n = 1
    if r.get("contract"):
        c = r["contract"]
        note = " ⚠ built from an older commit — rerun scripts/fetch_contracts.py" if c.get("stale") else ""
        L.append(f"{n}. **Contract** — [`{c['file']}`]({c['file']}): canonical LIDL, what codegen (C++/Rust/Nim) consumes.{note}")
        n += 1
    if r.get("interfaceSource"):
        i = r["interfaceSource"]
        L.append(f"{n}. **Annotated interface source** — [`{i['file']}`]({i['file']}), extracted from [`{i['from']}`]({i['url']}). Doc comments here are the API reference.")
        n += 1
    L.append(f"{n}. **Stack overview** — [stacks/{r['stack']}.md](../../stacks/{r['stack']}.md): how this module fits with its neighbours, trust boundaries, flows.")
    n += 1
    if r.get("notes"):
        L.append(f"{n}. **Curated notes** — [NOTES.md](NOTES.md).")
        n += 1
    if r.get("docs"):
        L.append(f"{n}. **Upstream docs** — " + ", ".join(f"[`{d['path']}`]({d['url']})" for d in r["docs"][:12]))
    api_ = r.get("api") or {}
    if api_.get("methods") or api_.get("events"):
        L += ["", "## API at a glance", ""]
        if api_.get("methods"):
            L += ["| Method | Signature | Summary |", "|---|---|---|"]
            for m in api_["methods"]:
                L.append(f"| `{m['name']}` | `{m['signature'].replace('|', '¦')}` | {m['summary'].replace('|', '¦')} |")
        if api_.get("events"):
            L += ["", "| Event | Payload | Summary |", "|---|---|---|"]
            for ev in api_["events"]:
                L.append(f"| `{ev['name']}` | `{ev['signature'].replace('|', '¦')}` | {ev['summary'].replace('|', '¦')} |")
    if r.get("provides") or r.get("uses"):
        L += ["", "## App-to-app intents", ""]
        for p in r.get("provides") or []:
            L.append(f"- provides `{p.get('action') or p.get('name') or p}`" if isinstance(p, dict) else f"- provides `{p}`")
        for u in r.get("uses") or []:
            L.append(f"- uses `{u}`")
    L += ["", "## Depend on it / get it", ""]
    if r["type"] == "core":
        L.append(f'- From your module\'s `metadata.json`: `"dependencies": ["{r["name"]}"]` (see [guides/calling-official-modules.md](../../guides/calling-official-modules.md)).')
    if src:
        flake = f"github:{src['repo']}/{src['commit']}" + (f"?dir={src['subdir']}" if src.get("subdir") else "")
        L.append(f"- Contract from source: `nix build '{flake}#lidl' --no-link --print-out-paths`")
    if cat:
        L.append(f"- Install: Basecamp → Package Manager, or `logosctl install {r['name']} --version {cat['latest']}`")
    return "\n".join(L) + "\n"


def render_llms(reg: dict) -> str:
    rel = reg["basecamp"]
    L = [
        "# Logos Module Atlas",
        "",
        f"> AI-readable registry of every Logos module available in Logos Basecamp {rel['tag']} "
        f"(latest release): {sum(m['availability']['bundledInBasecamp'] for m in reg['modules'])} bundled in the app, "
        f"{sum(m['availability']['inDefaultCatalog'] for m in reg['modules'])} installable from its default catalog. "
        "Each module has a card, its canonical LIDL contract, and its annotated interface source, all pinned to the exact source commit that was released.",
        "",
        "Start with AGENTS.md for how to use this repo. registry.json is the machine-readable form of everything below.",
        "",
        "## Start here",
        "",
        f"- [AGENTS.md]({RAW_BASE}/AGENTS.md): how an AI agent should navigate this registry",
        f"- [registry.json]({RAW_BASE}/registry.json): every module — versions, deps, source commit, API summary",
        f"- [Calling official modules]({RAW_BASE}/guides/calling-official-modules.md): depend on and call a module from C++, Rust, Nim or QML",
        f"- [Compatibility]({RAW_BASE}/guides/compatibility.md): SDK/builder generations and variants that must match",
        f"- [Key custody across chains]({RAW_BASE}/stacks/key-custody.md): who holds keys and how signing is approved, per chain",
        f"- [Gaps]({RAW_BASE}/gaps.md): roadmap items not shipped in {rel['tag']}",
        "",
        "## Stacks",
        "",
    ]
    for key, s in reg["stacks"].items():
        L.append(f"- [{s['title']}]({RAW_BASE}/stacks/{key}.md): {s['summary']}")
    by_stack: dict[str, list] = {}
    for m in reg["modules"]:
        by_stack.setdefault(m["stack"], []).append(m)
    for key, mods in by_stack.items():
        title = reg["stacks"].get(key, {}).get("title", key)
        L += ["", f"## Modules: {title}", ""]
        for m in mods:
            ver = (m.get("catalog") or {}).get("latest") or m["availability"].get("bundledVersion")
            L.append(f"- [{m['name']}]({RAW_BASE}/modules/{m['name']}/README.md): {m['type']} {ver} — {(m['description'] or '').strip()}")
            if m.get("contract"):
                L.append(f"  - contract: {RAW_BASE}/modules/{m['name']}/{m['contract']['file']}")
    return "\n".join(L) + "\n"


def splice_readme(reg: dict) -> None:
    path = ROOT / "README.md"
    if not path.exists():
        return
    text = path.read_text()
    start, end = "<!-- registry:start -->", "<!-- registry:end -->"
    if start not in text:
        return
    rel = reg["basecamp"]
    L = [
        start,
        f"_Generated {reg['generatedAt'][:10]} from Basecamp [{rel['tag']}]({rel['url']}) and its default catalog "
        f"[{reg['catalogs'][0]['displayName']}](https://github.com/{reg['catalogs'][0]['repo']}) "
        f"(index generated {(reg['catalogs'][0]['indexGeneratedAt'] or '')[:16]})._",
        "",
    ]
    by_stack: dict[str, list] = {}
    for m in reg["modules"]:
        by_stack.setdefault(m["stack"], []).append(m)
    for key, mods in by_stack.items():
        title = reg["stacks"].get(key, {}).get("title", key)
        L += [f"### [{title}](stacks/{key}.md)", "", "| Module | Type | Version | Where | Contract | Description |", "|---|---|---|---|---|---|"]
        for m in mods:
            av = m["availability"]
            where = " + ".join(x for x in ("bundled" if av["bundledInBasecamp"] else "", "catalog" if av["inDefaultCatalog"] else "") if x)
            ver = (m.get("catalog") or {}).get("latest") or av.get("bundledVersion") or ""
            con = f"[lidl](modules/{m['name']}/{m['contract']['file']})" if m.get("contract") else "—"
            desc = (m["description"] or "").replace("|", "¦")
            if len(desc) > 140:
                desc = desc[:137] + "…"
            L.append(f"| [`{m['name']}`](modules/{m['name']}/README.md) | {m['type']} | {ver} | {where} | {con} | {desc} |")
        L.append("")
    L.append(end)
    path.write_text(re.sub(re.escape(start) + r".*?" + re.escape(end), lambda _: "\n".join(L), text, flags=re.S))


def main() -> None:
    p = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    p.add_argument("--basecamp-tag", help="Basecamp release tag (default: the latest release)")
    build(p.parse_args())


if __name__ == "__main__":
    main()
