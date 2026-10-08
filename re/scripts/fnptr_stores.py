# List instructions that store a function address into memory at [reg + <offset>].
# Args: offset in hex (e.g. 0x8). Finds callback/hook installers.
# @runtime PyGhidra

from ghidra.program.model.scalar import Scalar

prog = currentProgram  # noqa: F821
fm = prog.getFunctionManager()
af = prog.getAddressFactory()
want = int(getScriptArgs()[0], 16)  # noqa: F821
for ins in prog.getListing().getInstructions(True):
    if not ins.getMnemonicString().startswith("MOV") or ins.getNumOperands() != 2:
        continue
    dst = ins.getDefaultOperandRepresentation(0)
    if not dst.endswith("+ 0x%x]" % want):
        continue
    for o in ins.getOpObjects(1):
        if isinstance(o, Scalar):
            try:
                target = fm.getFunctionAt(af.getAddress("%x" % o.getUnsignedValue()))
            except Exception:
                target = None
            if target:
                f = fm.getFunctionContaining(ins.getAddress())
                print("store %s in %s : %s -> %s" % (ins.getAddress(), f.getName() if f else "?", ins, target.getName()))
