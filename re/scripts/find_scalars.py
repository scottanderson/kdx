# Print every instruction using one of the given scalar values, with its function.
# Args: hex values, e.g. 0x2aa24aec 0x50585425
# @runtime PyGhidra

from ghidra.program.model.scalar import Scalar

prog = currentProgram  # noqa: F821
fm = prog.getFunctionManager()
wanted = {int(a, 16) & 0xFFFFFFFF for a in getScriptArgs()}  # noqa: F821
for ins in prog.getListing().getInstructions(True):
    for i in range(ins.getNumOperands()):
        for o in ins.getOpObjects(i):
            if isinstance(o, Scalar) and (o.getUnsignedValue() & 0xFFFFFFFF) in wanted:
                f = fm.getFunctionContaining(ins.getAddress())
                print("scalar 0x%x %s %s : %s" % (o.getUnsignedValue() & 0xFFFFFFFF, ins.getAddress(),
                                                 f.getName() + "@" + str(f.getEntryPoint()) if f else "?", ins))
