# Print each entry of a jump table, with the string its target loads. Args: <table> <count>
# @runtime PyGhidra

args = getScriptArgs()  # noqa: F821
prog = currentProgram  # noqa: F821
mem = prog.getMemory()
listing = prog.getListing()
table = toAddr(args[0])  # noqa: F821
count = int(args[1], 0)


def text_at(addr):
    """The C string at addr, or the length-prefixed one if the first byte is a plausible length."""
    raw = bytearray()
    for i in range(80):
        b = mem.getByte(addr.add(i)) & 0xff
        if b == 0:
            break
        raw.append(b)
    s = raw.decode("latin-1")
    if raw and raw[0] < 0x20 and raw[0] <= len(raw) - 1:
        s = raw[1:1 + raw[0]].decode("latin-1")
    return s


for i in range(count):
    target = toAddr(mem.getInt(table.add(4 * i)) & 0xffffffff)  # noqa: F821
    ins = listing.getInstructionAt(target)
    strs = []
    for _ in range(3):
        if ins is None:
            break
        for op in range(ins.getNumOperands()):
            for obj in ins.getOpObjects(op):
                if hasattr(obj, "getUnsignedValue") and obj.getUnsignedValue() > 0x08048000:
                    a = toAddr(obj.getUnsignedValue())  # noqa: F821
                    if mem.contains(a):
                        strs.append(text_at(a))
        if ins.getFlowType().isTerminal() or ins.getFlowType().isJump():
            break
        ins = ins.getNext()
    print("jt %d\t%s\t%s" % (i, target, " | ".join(strs)))
