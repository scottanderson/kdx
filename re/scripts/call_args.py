# Print each decompiled call to the given functions, with its arguments and calling function.
# Args: function addresses, e.g. 0x080856cc
# @runtime PyGhidra

from ghidra.app.decompiler import DecompInterface

prog = currentProgram  # noqa: F821
fm = prog.getFunctionManager()
di = DecompInterface()
di.openProgram(prog)
for arg in getScriptArgs():  # noqa: F821
    target = fm.getFunctionAt(prog.getAddressFactory().getAddress(arg.lower().replace("0x", "")))
    callers = {fm.getFunctionContaining(r.getFromAddress()) for r in prog.getReferenceManager().getReferencesTo(target.getEntryPoint())}
    for f in sorted((f for f in callers if f), key=lambda f: f.getEntryPoint()):
        res = di.decompileFunction(f, 60, monitor)  # noqa: F821
        if not res.decompileCompleted():
            print("call %s <- %s: decompile failed" % (arg, f.getEntryPoint()))
            continue
        for line in res.getDecompiledFunction().getC().split("\n"):
            if target.getName() + "(" in line:
                print("call %s <- %s: %s" % (arg, f.getEntryPoint(), line.strip()))
