# First-pass survey of a KDX binary: functions, network call sites, strings with xrefs,
# crypto-constant sites, and decompiles of the functions around them.
# Writes to /work/out/<program name>/.
# @runtime PyGhidra

import os
import re

from jpype import JArray, JByte

from ghidra.app.decompiler import DecompInterface
from ghidra.program.model.scalar import Scalar
from ghidra.util.task import ConsoleTaskMonitor

prog = currentProgram  # noqa: F821 (provided by Ghidra)
listing = prog.getListing()
fm = prog.getFunctionManager()
refman = prog.getReferenceManager()
mem = prog.getMemory()
monitor = ConsoleTaskMonitor()

OUT = os.path.join("/work/out", prog.getName().replace(".", "-"))
os.makedirs(os.path.join(OUT, "decomp"), exist_ok=True)

CONSTANTS = {
    0x4878: "tcp-xor-step",
    0xA5A16C4A: "udp-lcg-init",
    0x41D28485: "udp-lcg-mul",
    12843: "udp-lcg-add",
    0x67452301: "md5-init-a",
    0xEFCDAB89: "md5-init-b",
    0xD76AA478: "md5-t1",
    0xEDB88320: "crc32-poly",
    0x04C11DB7: "crc32-poly-normal",
}
NET_IMPORTS = {"socket", "bind", "listen", "accept", "connect", "send", "recv", "sendto",
               "recvfrom", "select", "poll", "setsockopt", "getsockopt", "shutdown", "close",
               "gethostbyname", "getpeername", "time", "rand", "srand", "random", "gettimeofday"}


def func_at(addr):
    f = fm.getFunctionContaining(addr)
    return f


def fname(f):
    return "%s@%s" % (f.getName(), f.getEntryPoint()) if f else "?"


def escape(s):
    return s.replace("\\", "\\\\").replace("\t", "\\t").replace("\n", "\\n").replace("\r", "\\r")


anchors = {}  # entry address string -> set(reasons)


def anchor(f, why):
    if f:
        anchors.setdefault(str(f.getEntryPoint()), set()).add(why)


# Functions
with open(os.path.join(OUT, "functions.tsv"), "w") as fh:
    fh.write("entry\tname\tsize\tcallers\tcallees\n")
    for f in fm.getFunctions(True):
        fh.write("%s\t%s\t%d\t%d\t%d\n" % (
            f.getEntryPoint(), f.getName(), f.getBody().getNumAddresses(),
            len(f.getCallingFunctions(monitor)), len(f.getCalledFunctions(monitor))))

# Callers of network/time imports
with open(os.path.join(OUT, "imports.tsv"), "w") as fh:
    fh.write("import\tcall_site\tfunction\n")
    for f in fm.getExternalFunctions():
        if f.getName() not in NET_IMPORTS:
            continue
        thunks = [f] + [fm.getFunctionAt(a) for a in (f.getFunctionThunkAddresses(True) or [])]
        for t in thunks:
            if t is None:
                continue
            for r in refman.getReferencesTo(t.getEntryPoint()):
                caller = func_at(r.getFromAddress())
                fh.write("%s\t%s\t%s\n" % (f.getName(), r.getFromAddress(), fname(caller)))
                if f.getName() in ("send", "recv", "sendto", "recvfrom", "accept", "connect"):
                    anchor(caller, "calls " + f.getName())

# Constant sites
with open(os.path.join(OUT, "constants.tsv"), "w") as fh:
    fh.write("constant\tlabel\taddress\tfunction\tinstruction\n")
    for ins in listing.getInstructions(True):
        for i in range(ins.getNumOperands()):
            for o in ins.getOpObjects(i):
                if isinstance(o, Scalar):
                    v = o.getUnsignedValue() & 0xFFFFFFFF
                    if v in CONSTANTS:
                        f = func_at(ins.getAddress())
                        fh.write("0x%x\t%s\t%s\t%s\t%s\n" % (v, CONSTANTS[v], ins.getAddress(), fname(f), ins))
                        anchor(f, CONSTANTS[v])

# Strings: Pascal (length-prefixed, Haxial's UI/error text) and C strings, with referencing functions
PRINTABLE = re.compile(rb"^[\x09\x0a\x0d\x20-\x7e\x80-\xff]+$")
INTERESTING = re.compile(r"protocol|breach|transaction|TRN|ID \^|encrypt|password|login|version|"
                         r"Contestant|story|transfer|tracker|chat|news|account|privilege", re.I)
with open(os.path.join(OUT, "strings.tsv"), "w") as fh:
    fh.write("address\tkind\tref_functions\ttext\n")
    for blk in mem.getBlocks():
        if blk.getName() not in (".rodata", ".data"):
            continue
        size = int(blk.getSize())
        jb = JArray(JByte)(size)
        blk.getBytes(blk.getStart(), jb)
        data = bytes(b & 0xFF for b in jb)
        for off in range(size):
            addr = blk.getStart().add(off)
            refs = [r for r in refman.getReferencesTo(addr)]
            if not refs:
                continue
            n = data[off]
            kind = None
            if n >= 3 and off + 1 + n <= size and PRINTABLE.match(data[off + 1:off + 1 + n]):
                kind, text = "pascal", data[off + 1:off + 1 + n]
            else:
                end = data.find(b"\x00", off)
                if end - off >= 4 and PRINTABLE.match(data[off:end]):
                    kind, text = "c", data[off:end]
            if not kind:
                continue
            funcs = sorted({fname(func_at(r.getFromAddress())) for r in refs})
            s = text.decode("latin-1")
            fh.write("%s\t%s\t%s\t%s\n" % (addr, kind, ",".join(funcs), escape(s)))
            if INTERESTING.search(s):
                for r in refs:
                    anchor(func_at(r.getFromAddress()), "str:" + s[:40])

# Decompile anchor functions
dec = DecompInterface()
dec.openProgram(prog)
with open(os.path.join(OUT, "anchors.tsv"), "w") as fh:
    fh.write("entry\tname\treasons\n")
    for entry, why in sorted(anchors.items()):
        f = fm.getFunctionAt(prog.getAddressFactory().getAddress(entry))
        fh.write("%s\t%s\t%s\n" % (entry, f.getName(), "; ".join(sorted(why))))
        res = dec.decompileFunction(f, 60, monitor)
        if res.decompileCompleted():
            with open(os.path.join(OUT, "decomp", "%s_%s.c" % (entry, f.getName())), "w") as out:
                out.write("// reasons: %s\n" % "; ".join(sorted(why)))
                out.write(res.getDecompiledFunction().getC())

print("overview: %d anchor functions written to %s" % (len(anchors), OUT))
