# Print the program's bytes at an address as hex, 16 to a line, such as a lookup table.
# Args: <addr> <count>, e.g. 0x0047fe98 256
# @runtime PyGhidra

prog = currentProgram  # noqa: F821
addr_arg, count_arg = getScriptArgs()  # noqa: F821
start = prog.getAddressFactory().getAddress(addr_arg.lower().replace("0x", ""))
count = int(count_arg, 0)
data = bytearray(b & 0xFF for b in getBytes(start, count))  # noqa: F821
for off in range(0, count, 16):
    print("bytes %s: %s" % (start.add(off), " ".join("%02x" % b for b in data[off:off + 16])))
