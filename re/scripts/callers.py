# List call sites of the given functions (address or name), with the calling function.
# @runtime PyGhidra

prog = currentProgram  # noqa: F821
fm = prog.getFunctionManager()
for arg in getScriptArgs():  # noqa: F821
    f = fm.getFunctionContaining(prog.getAddressFactory().getAddress(arg.lower().replace("0x", "")))
    for r in prog.getReferenceManager().getReferencesTo(f.getEntryPoint()):
        c = fm.getFunctionContaining(r.getFromAddress())
        print("caller %s <- %s in %s" % (f.getName(), r.getFromAddress(), c.getName() + "@" + str(c.getEntryPoint()) if c else "?"))
