# Decompile functions by entry/contained address or name to /work/out/<project>/decomp/.
# Args: one or more of 0x<addr> | <addr> | <function name>
# @runtime PyGhidra

import os
import sys

sys.path.insert(0, "/work/scripts")
from decomp_util import Decompiler  # noqa: E402

prog = currentProgram  # noqa: F821
fm = prog.getFunctionManager()
OUT = os.path.join("/work/out", prog.getDomainFile().getProjectLocator().getName(), "decomp")
os.makedirs(OUT, exist_ok=True)

dec = Decompiler(prog)
for arg in getScriptArgs():  # noqa: F821
    f = None
    try:
        f = fm.getFunctionContaining(prog.getAddressFactory().getAddress(arg.lower().replace("0x", "")))
    except Exception:
        pass
    if f is None:
        f = next(iter(getGlobalFunctions(arg)), None)  # noqa: F821
    if f is None:
        print("decompile: no function for %s" % arg)
        continue
    path = os.path.join(OUT, "%s_%s.c" % (f.getEntryPoint(), f.getName()))
    with open(path, "w") as out:
        out.write(dec.decompile(f, 120))
    print("decompile: %s -> %s" % (arg, os.path.basename(path)))
