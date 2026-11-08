"""Print a PEF build's message type -> name table from its decompiled code.

Usage:
  python3 -I re/name_table.py ALL_C FUNC BINARY ANCHOR_NAME ANCHOR_TYPE
  python3 -I re/name_table.py --toc ALL_C FUNC BINARY FIRST_NAME

The PowerPC Mac builds name each message type with a function that is a tree
of `if (param_1 == N)` tests. FUNC is its address in `re/out/<project>/all.c`,
as decompile_all.py writes it, such as 1000ff90.

Default mode: each branch returns `base + offset`, where base points at a
block of Pascal strings. ANCHOR_NAME and ANCHOR_TYPE give one known pair, such
as "Server History Added" 183, which fixes the block in BINARY.

--toc mode (0.9x builds): each branch returns a TOC slot, one slot per name,
in the order of a block of Pascal strings that starts with FIRST_NAME, such as
"Set Identity". Type 100 must be the first name.

Prints `type name` lines in type order, and leaves out the default name.
"""

import re
import sys


def func_text(path, addr):
    out, on = [], False
    for line in open(path, encoding="utf-8", errors="replace"):
        if line.startswith("//// "):
            if on:
                break
            on = line.split()[1] == addr
        if on:
            out.append(line)
    body = "".join(out)
    return body[body.index("{", body.index("(")) :]


def to_python(c):
    """Turns the if/else tree into the body of a Python function."""
    c = re.sub(r"/\*.*?\*/", "", c, flags=re.S)
    c = re.sub(r"\((?:ulonglong|longlong|int|uint)\)", "", c)
    toks = re.findall(r"else if|if|else|\{|\}|;|\(|\)|[^\s{}();]+", c)
    lines, ind, i, cur = [], 1, 0, ""
    while i < len(toks):
        t = toks[i]
        if t in ("if", "else if"):
            depth, j, cond = 0, i + 1, []
            while True:
                cond.append(toks[j])
                depth += {"(": 1, ")": -1}.get(toks[j], 0)
                if depth == 0:
                    break
                j += 1
            lines.append("  " * ind + ("if " if t == "if" else "elif ") + " ".join(cond) + ":")
            i = j + 1
            continue
        if t == "else":
            lines.append("  " * ind + "else:")
        elif t == "{":
            ind += 1
            lines.append("  " * ind + "pass")
        elif t == "}":
            ind -= 1
        elif t == ";":
            stmt = cur.strip()
            # Keep returns and assignments; drop declarations such as "uint uVar1".
            if stmt.startswith("return") or "=" in stmt:
                lines.append("  " * ind + stmt)
            cur = ""
        else:
            cur += " " + t
        i += 1
    py = "\n".join(lines).replace("&&", " and ").replace("||", " or ")
    return "def f(param_1):\n" + py + "\n"


def compile_tree(path, addr, toc):
    c = func_text(path, addr)
    if toc:
        c = re.sub(r"return _?DAT_([0-9a-f]+);", r"return 0x\1;", c)
    else:
        # The string base is the first variable loaded from a global.
        c = re.sub(r"(\w+) = [^;]*(?:DAT_|PTR_)[^;]*;", r"\1 = 0;", c, count=1)
    ns = {}
    exec(to_python(c), ns)
    return ns["f"]


def pascal_strings(data, start):
    out = []
    while start < len(data):
        n = data[start]
        s = data[start + 1 : start + 1 + n]
        if n == 0 or not all(32 <= b < 127 for b in s):
            break
        out.append(s.decode("ascii"))
        start += 1 + n
    return out


def main():
    args = sys.argv[1:]
    toc = args[:1] == ["--toc"]
    if toc:
        args = args[1:]
    path, addr, binary = args[:3]
    f = compile_tree(path, addr, toc)
    data = open(binary, "rb").read()
    found = {}
    if toc:
        first = args[3].encode()
        names = pascal_strings(data, data.index(bytes([len(first)]) + first))
        base = f(100)
        for t in range(1, 5000):
            v = f(t)
            if v is not None:
                idx = (base - v) // 4
                found.setdefault(v, []).append((t, names[idx] if 0 <= idx < len(names) else "?"))
    else:
        anchor, atype = args[3].encode(), int(args[4])
        base = data.index(bytes([len(anchor)]) + anchor) - f(atype)
        for t in range(1, 5000):
            p = base + f(t)
            found.setdefault(p, []).append((t, pascal_strings(data, p)[0]))
    default = max(found.values(), key=len)
    for items in sorted(found.values()):
        if items is not default:
            for t, name in items:
                print(t, name)


if __name__ == "__main__":
    main()
