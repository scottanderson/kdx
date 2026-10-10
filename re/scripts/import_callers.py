# List call sites of imported functions by name. Args: <name>...
# @runtime PyGhidra

prog = currentProgram  # noqa: F821
fm = prog.getFunctionManager()
refs = prog.getReferenceManager()
for name in getScriptArgs():  # noqa: F821
    for sym in prog.getSymbolTable().getSymbols(name):
        for r in refs.getReferencesTo(sym.getAddress()):
            c = fm.getFunctionContaining(r.getFromAddress())
            print("import %s <- %s in %s" % (name, r.getFromAddress(), "%s@%s" % (c.getName(), c.getEntryPoint()) if c else "?"))
