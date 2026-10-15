#!/usr/bin/env python3
"""Runs CI checks over the Markdown docs."""
import collections
import glob
import os
import re
import sys

SKIP = ("vendor/", "target/", "re/out/", "re/projects/", ".git/")
ID = re.compile(r'<a id="([^"]+)"></a>')
errors = []
edges = collections.defaultdict(set)


def slug(heading):
    text = re.sub(r"<[^>]*>", "", heading).lower()
    return re.sub(r"[^\w\- ]", "", text).replace(" ", "-")


def anchors(path, cache={}):
    if path not in cache:
        found = set()
        fence = False
        for line in open(path, encoding="utf-8"):
            if line.startswith("```"):
                fence = not fence
            if fence:
                continue
            if re.match(r"^#+ ", line):
                found.add(slug(re.sub(r"^#+ ", "", line.rstrip("\n"))))
            found.update(ID.findall(line))
        cache[path] = found
    return cache[path]


def markdown_files():
    for path in sorted(glob.glob("**/*.md", recursive=True)):
        if not path.startswith(SKIP) and not path.endswith("/CLAUDE.md"):
            yield path


def check_links():
    for path in markdown_files():
        fence = False
        for n, line in enumerate(open(path, encoding="utf-8"), 1):
            if line.startswith("```"):
                fence = not fence
            if fence:
                continue
            for m in re.finditer(r"\]\(([^)\s]*)\)", line):
                target, _, frag = m.group(1).partition("#")
                if "github.com/scottanderson/kdx" in target and not re.fullmatch(r"crates/[^/]+/README\.md", path):
                    errors.append(f"{path}:{n}: use a relative link, not {m.group(1)}")
                if re.match(r"[a-z]+:", target):
                    continue
                if path.startswith("crates/") and not target:
                    continue
                dest = os.path.normpath(os.path.join(os.path.dirname(path), target)) if target else path
                if dest != path:
                    edges[path].add(dest)
                if not os.path.exists(dest):
                    errors.append(f"{path}:{n}: missing file {m.group(1)}")
                elif frag and dest.endswith(".md") and frag not in anchors(dest):
                    errors.append(f"{path}:{n}: missing anchor {m.group(1)}")


PATH = re.compile(r"`([\w./-]+)`")
KNOWN = (".md", ".rs", ".py", ".toml", ".yml", ".sh", ".json", ".pcapng", ".pcap", ".lock")
# Ignored, generated, or in the lab guest.
EXEMPT = ("vendor/haxial/", "vendor/refs/", "target/", "re/out/", "re/projects/", ".git/", "lab/run/", "Bases/", "lab-files/")


def check_orphans():
    # Every page must be reachable from README.md by following links.
    seen = {"README.md"}
    todo = ["README.md"]
    while todo:
        for dest in edges[todo.pop()]:
            if dest not in seen:
                seen.add(dest)
                todo.append(dest)
    for path in markdown_files():
        if path not in seen:
            errors.append(f"{path}: not reachable from README.md")


def check_paths():
    # Backticked repo paths must exist.
    tops = {d for d in os.listdir(".") if os.path.isdir(d) and not d.startswith(".")}
    for path in tracked_markdown():
        fence = False
        for n, line in enumerate(open(path, encoding="utf-8"), 1):
            if line.startswith("```"):
                fence = not fence
            if fence:
                continue
            for ref in PATH.findall(line):
                if ref.startswith(("/", "./", "../")) or ref.startswith(EXEMPT):
                    continue
                if "/" not in ref:
                    # A bare file name is in the repository root or re/scripts/.
                    found = not ref.endswith(KNOWN) or os.path.exists(ref) or os.path.exists("re/scripts/" + ref)
                elif ref.endswith(KNOWN) or ref.endswith("/") or ref.split("/")[0] in tops:
                    found = exists_near(path, ref)
                else:
                    found = True
                if not found:
                    errors.append(f"{path}:{n}: no such path `{ref}`")


def exists_near(page, ref):
    # Resolve from the page's directory and each parent, then the repository root.
    where = os.path.dirname(page)
    while where:
        if os.path.exists(os.path.join(where, ref)):
            return True
        where = os.path.dirname(where)
    return os.path.exists(ref)


def tracked_markdown():
    return [p for p in markdown_files() if not p.startswith("captures/") or p == "captures/README.md"]


def check_message_anchors():
    seen = collections.defaultdict(list)
    pages = glob.glob("docs/protocol/messages/*.md") + ["docs/protocol/dcc.md"]
    for path in sorted(pages):
        ids = []
        for n, line in enumerate(open(path, encoding="utf-8"), 1):
            for name in ID.findall(line):
                if not re.fullmatch(r"m\d+", name):
                    continue
                seen[name].append(path)
                ids.append(int(name[1:]))
                if not line.startswith("#"):
                    errors.append(f"{path}:{n}: {name} is not on a heading")
        if ids != sorted(ids):
            errors.append(f"{path}: message headings are not in ascending order")
    for name, where in seen.items():
        if len(where) > 1:
            errors.append(f"{name} is anchored more than once: {where}")
    constants = re.findall(
        r'^\s+[A-Z][A-Z_0-9]* = (\d+), "', open("crates/kdx-proto/src/message_type.rs", encoding="utf-8").read(), re.M
    )
    for value in constants:
        if f"m{value}" not in seen:
            errors.append(f"message type {value} has no #m{value} anchor in docs/protocol")


def main():
    os.chdir(os.path.join(os.path.dirname(__file__), ".."))
    check_links()
    check_orphans()
    check_paths()
    check_message_anchors()
    for error in errors:
        print(error)
    if errors:
        sys.exit(1)
    print("docs ok")


main()
