#!/usr/bin/env python3
"""Checks the Markdown docs: links, anchors, and the message type anchors."""
import collections
import glob
import os
import re
import sys

SKIP = ("vendor/", "target/", "re/out/", "re/projects/", ".git/")
ID = re.compile(r'<a id="([^"]+)"></a>')
errors = []


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
                if not os.path.exists(dest):
                    errors.append(f"{path}:{n}: missing file {m.group(1)}")
                elif frag and dest.endswith(".md") and frag not in anchors(dest):
                    errors.append(f"{path}:{n}: missing anchor {m.group(1)}")


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
    check_message_anchors()
    for error in errors:
        print(error)
    if errors:
        sys.exit(1)
    print("docs ok")


main()
