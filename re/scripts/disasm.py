# Print instructions from a start address. Args: <start> [count=40]
# @runtime PyGhidra

args = getScriptArgs()  # noqa: F821
prog = currentProgram  # noqa: F821
listing = prog.getListing()
addr = toAddr(args[0])  # noqa: F821
count = int(args[1]) if len(args) > 1 else 40
ins = listing.getInstructionAt(addr) or listing.getInstructionAfter(addr)
for _ in range(count):
    if ins is None:
        break
    refs = [str(r.getToAddress()) for r in ins.getReferencesFrom()]
    print("asm %s  %-40s %s" % (ins.getAddress(), ins, " ".join(refs)))
    ins = ins.getNext()
