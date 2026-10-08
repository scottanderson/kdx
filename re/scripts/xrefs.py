# Find each occurrence of a byte string and print what references it (and what references those).
# Args: text [text...]
# @runtime PyGhidra

prog = currentProgram  # noqa: F821
mem = prog.getMemory()
fm = prog.getFunctionManager()
rm = prog.getReferenceManager()


def where(addr):
    f = fm.getFunctionContaining(addr)
    return "%s in %s" % (addr, f.getName() + "@" + str(f.getEntryPoint()) if f else "data")


for text in getScriptArgs():  # noqa: F821
    needle = bytes(text, "latin-1")
    addr = mem.getMinAddress()
    while True:
        addr = mem.findBytes(addr, needle, None, True, monitor)  # noqa: F821
        if addr is None:
            break
        print("xref %r at %s" % (text, addr))
        for start in (addr, addr.subtract(1)):
            for r in rm.getReferencesTo(start):
                print("xref   <- %s" % where(r.getFromAddress()))
                for r2 in rm.getReferencesTo(r.getFromAddress()):
                    print("xref      <- %s" % where(r2.getFromAddress()))
        addr = addr.add(1)
