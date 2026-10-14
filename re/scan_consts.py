"""Report which header scramble constants each binary contains.

Usage: python3 -I re/scan_consts.py FILE...  (skips anything not a file)

Finds x86 builds' constants as little-endian immediates. Finds PowerPC
builds' constants as a `lis`/`addis`/`xoris` high half within six
instructions of an `addi`/`ori`/`xori` low half, which is how PEF and Mach-O
builds load them. Prints one line per file: `kdx=` and `hgc=` with a `1` for
each of the four constants found, in the order `framing.md` and `gconsole.md`
list them.
"""

import os
import struct
import sys

CONSTS = {
    "kdx": [0x6E7DFD34, 0xE152DA04, 0xA95A759B, 0x06992E25],
    "hgc": [0x5F547A17, 0x69C83E35, 0x1B20E200, 0x8022E8D1],
}


def ppc_consts(data):
    n = len(data) // 4
    words = struct.unpack(">%dI" % n, data[: n * 4])
    found = set()
    for i, w in enumerate(words):
        if w >> 26 not in (15, 27):  # addis (lis), xoris
            continue
        hi = w & 0xFFFF
        for v in words[max(0, i - 6) : i + 7]:
            op = v >> 26
            if op not in (14, 24, 26):  # addi, ori, xori
                continue
            lo = v & 0xFFFF
            found.add((hi << 16) | lo)
            if op == 14 and lo & 0x8000:  # addi sign-extends
                found.add(((hi << 16) + lo - 0x10000) & 0xFFFFFFFF)
    return found


def main():
    for path in sys.argv[1:]:
        if not os.path.isfile(path):
            continue
        with open(path, "rb") as f:
            data = f.read()
        if data[:2] == b"MZ" or data[:4] == b"\x7fELF":
            has = lambda v: struct.pack("<I", v) in data  # noqa: E731
        else:
            found = ppc_consts(data)
            has = found.__contains__
        row = " ".join(
            name + "=" + "".join("1" if has(v) else "." for v in vs)
            for name, vs in CONSTS.items()
        )
        print(row, path)


if __name__ == "__main__":
    main()
