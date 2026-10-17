# Decompile every function to /work/out/<project>/all.c, one block per function, for grepping.
# @runtime PyGhidra

import os
import sys

sys.path.insert(0, "/work/scripts")
from decomp_util import Decompiler  # noqa: E402

prog = currentProgram  # noqa: F821
OUT = os.path.join("/work/out", prog.getDomainFile().getProjectLocator().getName())
os.makedirs(OUT, exist_ok=True)

dec = Decompiler(prog)
n = 0
with open(os.path.join(OUT, "all.c"), "w") as out:
    for f in prog.getFunctionManager().getFunctions(True):
        if f.isThunk() or f.isExternal():
            continue
        out.write("\n//// %s %s\n" % (f.getEntryPoint(), f.getName()))
        out.write(dec.decompile(f, 60))
        n += 1
print("decompile_all: %d functions" % n)
