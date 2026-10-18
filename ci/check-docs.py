#!/usr/bin/env python3
"""Runs CI checks over the Markdown docs."""
import collections
import glob
import os
import re
import subprocess
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
    listed = subprocess.run(
        ["git", "ls-files", "--cached", "--others", "--exclude-standard", "*.md"],
        capture_output=True, text=True, check=True,
    ).stdout.split()
    for path in sorted(p for p in listed if not p.startswith(".")):
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


def check_conventions_link():
    # Every protocol page except the README links to the conventions in its first 5 lines.
    pages = glob.glob("docs/protocol/*.md") + glob.glob("docs/protocol/messages/*.md")
    for path in sorted(pages):
        head = open(path, encoding="utf-8").read().split("\n")[:5]
        if path != "docs/protocol/README.md" and not any("README.md#conventions)" in line for line in head):
            errors.append(f"{path}: no link to the protocol README conventions in the first 5 lines")


def check_one_h1():
    for path in tracked_markdown():
        fence = False
        count = 0
        for line in open(path, encoding="utf-8"):
            if line.startswith("```"):
                fence = not fence
            elif not fence and line.startswith("# "):
                count += 1
        if count != 1:
            errors.append(f"{path}: {count} H1 headings, expected 1")


def cells(line):
    return len(re.split(r"(?<!\\)\|", line.strip().strip("|")))


def check_table_columns():
    # GFM renders a table only when the delimiter row matches the header.
    for path in tracked_markdown():
        fence = False
        prev = ""
        for n, line in enumerate(open(path, encoding="utf-8"), 1):
            if line.startswith("```"):
                fence = not fence
            elif not fence and re.fullmatch(r"\|(?: *:?-+:? *\|)+\s*", line) and prev.startswith("|"):
                if cells(line) != cells(prev):
                    errors.append(f"{path}:{n}: delimiter row has {cells(line)} columns, header has {cells(prev)}")
            prev = line


TAG = re.compile(r"(?<=<!-- )\[[OSI][^\]]*\]")
TAG_O = r"O(?: `captures/[\w-]+\.pcapng`)?(?: frames? \d+(?: to \d+)?(?:, \d+(?: to \d+)?)*)?"

TAG_S = r"S(?: (?:(?:client|server|tracker) )?`[^`]+`(?:, (?:(?:client|server|tracker) )?`[^`]+`)*)?"


def check_tags():
    # Evidence tags: S, I, O in that order, joined by "; ", no remarks.
    for path in markdown_files():
        text = open(path, encoding="utf-8").read()
        for m in re.finditer(r"\*\*\[[OSI][^\]]*\]\*\*", text):
            line = text.count("\n", 0, m.start()) + 1
            errors.append(f"{path}:{line}: put the evidence tag in an HTML comment: {m.group(0)!r}")
        if not path.startswith("docs/"):
            continue
        for m in TAG.finditer(text):
            inner = " ".join(m.group(0)[1:-1].split())
            parts = re.split(r"\s*;\s*", inner)
            kinds = [p[:1] for p in parts]
            good = (
                "; ".join(parts) == inner
                and kinds == sorted(kinds, key=lambda k: "SIO".find(k))
                and kinds.count("S") <= 1
                and kinds.count("I") <= 1
            )
            for part in parts:
                kind = part[:1]
                if kind == "O" and not re.fullmatch(TAG_O, part):
                    good = False
                if kind == "S" and not re.fullmatch(TAG_S, part):
                    good = False
                if kind == "I" and part != "I":
                    good = False
            if not good:
                line = text.count("\n", 0, m.start()) + 1
                errors.append(f"{path}:{line}: evidence tag does not follow the grammar: {m.group(0)!r}")


def check_open_questions():
    # An Open questions section lists only open items.
    for path in tracked_markdown():
        inside = False
        fence = False
        for n, line in enumerate(open(path, encoding="utf-8"), 1):
            if line.startswith("```"):
                fence = not fence
            if fence:
                continue
            if re.match(r"^#+ ", line):
                inside = "open questions" in line.lower()
            elif inside and re.match(r"\s*[-*] Resolved\b", line):
                errors.append(f"{path}:{n}: resolved item in an Open questions section")

# uN and iN take any whole number of bytes, as in LLVM: u24, u48, u128.
INTEGER = re.compile(r"^[ui]([1-9][0-9]*)$")


def primitive_size(base):
    # Returns the size in bytes of a primitive type, or None.
    if base == "char":
        return 1
    m = INTEGER.match(base)
    if m and int(m.group(1)) % 8 == 0:
        return int(m.group(1)) // 8
    return None

STRUCT_HEAD = "| Off | Size | Type | Field | Notes |"
STRUCT_SEP = "|---|---|---|---|---|"
CAPTION = re.compile(r'^Struct `([A-Z][A-Za-z0-9]*)` \((?:(\d+) bytes|dynamic)\):<a id="s\1"></a>$')
TYPE = re.compile(r"^([A-Za-z][A-Za-z0-9]*)(?:\[([^\]]*)\])?$")
INT = re.compile(r"^(?:\d+|0x[0-9a-fA-F]+)$")


def struct_pages():
    return sorted(glob.glob("docs/protocol/*.md") + glob.glob("docs/protocol/messages/*.md"))


def read_lines(path):
    # Yields (line number, text) outside code fences.
    fence = False
    for n, line in enumerate(open(path, encoding="utf-8"), 1):
        line = line.rstrip("\n")
        if line.startswith("```"):
            fence = not fence
            continue
        if not fence:
            yield n, line


def find_tables(path):
    # Returns (caption line, caption text, [(line, row text)], blank after caption) for each caption.
    lines = list(read_lines(path))
    found = []
    for i, (n, text) in enumerate(lines):
        if not text.startswith("Struct `"):
            continue
        rows = []
        j = i + 1
        if j < len(lines) and lines[j][1] == "":
            j += 1
        while j < len(lines) and lines[j][1].startswith("|"):
            rows.append(lines[j])
            j += 1
        found.append((n, text, rows, i + 1 < len(lines) and lines[i + 1][1] == ""))
        if rows and j < len(lines) and lines[j][1] != "":
            errors.append(f"{path}:{lines[j][0]}: put a blank line after the struct table")
    return found


def check_captions(path):
    # A table with the struct header row needs a struct caption.
    lines = list(read_lines(path))
    for i, (n, text) in enumerate(lines):
        if text != STRUCT_HEAD:
            continue
        before = [t for _, t in lines[max(0, i - 2) : i]]
        if not any(t.startswith("Struct `") for t in before):
            errors.append(f"{path}:{n}: struct table has no caption")


def check_layout_tables(path):
    # A table that starts with an Off column or a 0x00 row lays out bytes, so it must be a struct table.
    lines = list(read_lines(path))
    for i, (n, text) in enumerate(lines[:-1]):
        if not text.startswith("|") or not re.match(r"^\|\s*:?-", lines[i + 1][1]):
            continue
        if i > 0 and lines[i - 1][1].startswith("|"):
            continue
        first = text.strip("|").split("|")[0].strip().lower()
        row = lines[i + 2][1] if i + 2 < len(lines) else ""
        starts = re.match(r"^\|\s*`?0x0+`?\s*\|", row)
        if (first in ("off", "offset") or starts) and text != STRUCT_HEAD:
            errors.append(f"{path}:{n}: a byte layout must be a struct table with header {STRUCT_HEAD!r}")


def check_structs():
    structs = {}
    parsed = []
    for path in tracked_markdown():
        if path.startswith("docs/"):
            check_layout_tables(path)
    for path in struct_pages():
        found = find_tables(path)
        check_captions(path)
        for n, text, rows, blank in found:
            m = CAPTION.match(text)
            if not m:
                errors.append(f"{path}:{n}: bad struct caption {text!r}")
                continue
            name = m.group(1)
            size = int(m.group(2)) if m.group(2) is not None else None
            if name in structs:
                errors.append(f"{path}:{n}: struct {name} is already defined at {structs[name][0]}")
                continue
            structs[name] = (f"{path}:{n}", size)
            parsed.append((path, n, name, size, rows, blank))
    for path, n, name, size, rows, blank in parsed:
        check_struct(path, n, name, size, rows, blank, structs)
    return structs


FIELD = re.compile(r"^[a-z][a-z0-9_]*$")
DOC_ONLY = re.compile(r"^(?:ignored(?:_\w+)?|reserved(?:_\w+)?|padding|\w+_(?:len|count))$")
RUST_REF = re.compile(r"Rust `(\w+)(?:[.\[][^`]*)?`")
NOT_STORED = "not stored."


def rust_structs():
    # Maps each struct in kdx-proto to its pub field names.
    found = {}
    for path in glob.glob("crates/kdx-proto/src/**/*.rs", recursive=True):
        text = open(path, encoding="utf-8").read()
        for m in re.finditer(r"pub struct (\w+) \{(.*?)\n\}", text, re.S):
            found.setdefault(m.group(1), re.findall(r"^\s*pub (\w+):", m.group(2), re.M))
    return found


def check_rust_fields(err, n, name, rows, lengths, rust):
    # A row Notes may say "Rust `field`" for a Rust field it holds part of, or "not stored.".
    if name not in rust:
        return
    fields = [r[0] for r in rows]
    covered = set(fields)
    for _, notes in ((r[0], r[1]) for r in rows):
        covered.update(RUST_REF.findall(notes))
    for f in rust[name]:
        if f not in covered:
            err(n, f"Rust field {f!r} is missing from the table")
    for field, notes in rows:
        if field in rust[name] or field in lengths or DOC_ONLY.match(field):
            continue
        refs = RUST_REF.findall(notes)
        if notes.startswith(NOT_STORED) or any(r in rust[name] for r in refs):
            continue
        err(n, f"Field {field!r} is not a field of the Rust type; say \"Rust `field`\" or \"not stored.\" in Notes")


RUST = rust_structs()


def check_struct(path, n, name, size, rows, blank, structs):
    def err(line, msg):
        errors.append(f"{path}:{line}: struct {name}: {msg}")

    if not blank:
        err(n, "put a blank line between the caption and the table")
    if len(rows) < 3:
        err(n, "the table has no rows")
        return
    if rows[0][1] != STRUCT_HEAD:
        err(rows[0][0], f"header must be exactly {STRUCT_HEAD!r}")
    if rows[1][1] != STRUCT_SEP:
        err(rows[1][0], f"separator must be exactly {STRUCT_SEP!r}")
    expect = 0
    dynamic = False
    fields = []
    named = []
    lengths = set()
    widths = {}
    seen_rest = False
    for line, text in rows[2:]:
        cells = [c.strip() for c in text.strip()[1:-1].split("|")] if text.rstrip().endswith("|") else []
        if len(cells) != 5:
            err(line, "a row needs exactly 5 cells")
            continue
        off, sz, typ, field, notes = cells
        if re.fullmatch(r"`[^`]+`", field):
            field = field[1:-1]
        else:
            err(line, f"wrap Field {field!r} in backticks")
        if re.fullmatch(r"`[^`]+`", sz):
            sz = sz[1:-1]
        elif re.fullmatch(r"[a-z][a-z0-9_]*", sz) and sz not in ("rest", "pad4"):
            err(line, f"wrap Size {sz!r} in backticks")
        named.append((field, notes))
        if seen_rest:
            err(line, "no row may follow a rest row")
        if not FIELD.match(field):
            err(line, f"Field {field!r} must be a snake_case name")
        elif field in fields:
            err(line, f"duplicate Field name {field!r}")
        if sz == "rest":
            seen_rest = True
        if off and not re.fullmatch(r"0x[0-9a-f]+", off):
            err(line, f"bad Off {off!r}: use lowercase 0x hex")
        if sz and not re.fullmatch(r"\d+|rest|pad4|[a-z][a-z0-9_]*", sz):
            err(line, f"bad Size {sz!r}: use decimal bytes, rest, pad4, or a Field name")
        if dynamic and off:
            err(line, "Off must be empty after a dynamic row")
        if not dynamic:
            if not off:
                err(line, "Off is empty before any dynamic row")
            elif re.fullmatch(r"0x[0-9a-f]+", off) and int(off, 16) != expect:
                err(line, f"Off {off} should be {hex(expect)}")
        if typ == "char[]":
            err(line, "use str for text of variable size")
        link = re.fullmatch(r"\[(\w+)\]\(([^)]*)\)(.*)", typ)
        if link:
            typ = link.group(1) + link.group(3)
            want = os.path.relpath(structs[link.group(1)][0].split(":")[0], os.path.dirname(path)) if link.group(1) in structs else None
            if want is not None and link.group(2) != (f"#s{link.group(1)}" if want == os.path.basename(path) else f"{want}#s{link.group(1)}"):
                err(line, f"link for {link.group(1)} should point to its caption")
        m = TYPE.match(typ)
        if m and m.group(1) in structs and not link:
            err(line, f"link struct type {m.group(1)} to its caption")
        if m and link and m.group(1) not in structs:
            err(line, f"{m.group(1)!r} is not a struct; leave it unlinked")
        if not m:
            err(line, f"bad Type {typ!r}")
            fields.append(field)
            dynamic = dynamic or not sz
            continue
        base, count = m.groups()
        unit = None
        if base == "str":
            # Text is bytes: a bare str is variable, str[N] is N variable strings.
            unit = 1
            unit_dynamic = True
            if count is None:
                count = ""
        elif primitive_size(base):
            unit = primitive_size(base)
            unit_dynamic = False
        elif base in structs:
            unit = structs[base][1]
            unit_dynamic = unit is None
        else:
            err(line, f"unknown type {base!r}")
            unit_dynamic = False
        fixed = re.fullmatch(r"\d+", sz) is not None
        if count == "":
            # A variable array: Size is rest, pad4, or the Field that counts it.
            if fixed or not sz:
                err(line, "Type [] needs a Size of rest, pad4, or a Field name")
        elif count is not None and not INT.match(count):
            err(line, f"count {count!r} must be a number or empty")
        if sz and not fixed and sz not in ("rest", "pad4"):
            if sz not in fields:
                err(line, f"Size {sz!r} is not rest, pad4, or an earlier Field")
            else:
                lengths.add(sz)
                counts_elements = base != "str" and base != "u8" and count == ""
                want = f"{field}_{'count' if counts_elements else 'len'}"
                if sz != want:
                    err(line, f"Size {sz!r} should be {want!r}")
                width = widths.get(sz)
                if base == "str" and count not in ("", None):
                    if width != int(count, 0):
                        err(line, f"Size {sz!r} must be an array of {count} lengths")
                elif width is not None:
                    err(line, f"Size {sz!r} names an array of lengths; the row must be str[{width}]")
        if fixed:
            mult = int(count, 0) if count else 1
            if count == "":
                pass
            elif unit_dynamic:
                err(line, f"Size must not be a number for dynamic struct {base}")
            elif unit is not None and int(sz) != unit * mult:
                err(line, f"Size {sz} should be {unit * mult}")
            if not dynamic and count != "" and not unit_dynamic and unit is not None:
                expect += int(sz)
        else:
            if not sz and count is not None:
                pass
            elif not sz and not unit_dynamic and unit is not None:
                err(line, "Size is empty but the row is fixed size; give a Size or a dynamic struct")
            dynamic = True
        fields.append(field)
        if count and INT.match(count):
            widths[field] = int(count, 0)
    if size is not None and dynamic:
        err(n, "caption says bytes but the table has a dynamic row")
    elif size is None and not dynamic:
        err(n, "caption says dynamic but every row is fixed size")
    elif size is not None and expect != size:
        err(n, f"rows add up to {expect} bytes, caption says {size}")
    check_rust_fields(err, n, name, named, lengths, RUST)


def check_message_sections(structs):
    for path in sorted(glob.glob("docs/protocol/messages/*.md")):
        if path.endswith("/README.md"):
            continue
        lines = list(read_lines(path))
        heads = [i for i, (n, t) in enumerate(lines) if re.match(r"#{1,3} ", t)]
        for k, i in enumerate(heads):
            n, text = lines[i]
            anchors_ = [a for a in ID.findall(text) if re.fullmatch(r"m\d+", a)]
            if len(anchors_) != 1 or not text.startswith(("## ", "### ")):
                continue
            end = heads[k + 1] if k + 1 < len(heads) else len(lines)
            body = [t for _, t in lines[i + 1 : end]]
            has_caption = any(t.startswith("Struct `") for t in body)
            if not has_caption and "No payload." not in body:
                errors.append(f"{path}:{n}: {anchors_[0]} needs a struct table or a line 'No payload.'")
            for ln, t in lines[i + 1 : end]:
                if t.startswith("| Off |") and t != STRUCT_HEAD:
                    errors.append(f"{path}:{ln}: layout table is not in the struct shape")


def main():
    os.chdir(os.path.join(os.path.dirname(__file__), ".."))
    check_links()
    check_orphans()
    check_paths()
    check_message_anchors()
    check_conventions_link()
    check_one_h1()
    check_table_columns()
    check_tags()
    check_open_questions()
    check_message_sections(check_structs())
    for error in errors:
        print(error)
    if errors:
        sys.exit(1)
    print("docs ok")


main()
