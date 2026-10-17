# Decompile every function to /work/out/<project>/all.c, one block per function, for grepping.
# @runtime PyGhidra

import os

from ghidra.app.decompiler import DecompInterface
from ghidra.util.task import ConsoleTaskMonitor

prog = currentProgram  # noqa: F821
OUT = os.path.join("/work/out", prog.getDomainFile().getProjectLocator().getName())
os.makedirs(OUT, exist_ok=True)

dec = DecompInterface()
dec.openProgram(prog)
mon = ConsoleTaskMonitor()
n = 0
with open(os.path.join(OUT, "all.c"), "w") as out:
    for f in prog.getFunctionManager().getFunctions(True):
        if f.isThunk() or f.isExternal():
            continue
        res = dec.decompileFunction(f, 60, mon)
        out.write("\n//// %s %s\n" % (f.getEntryPoint(), f.getName()))
        out.write(res.getDecompiledFunction().getC() if res.decompileCompleted() else "// failed: %s\n" % res.getErrorMessage())
        n += 1
print("decompile_all: %d functions" % n)
