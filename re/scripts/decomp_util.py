# Shared by decompile.py and decompile_all.py: a decompile that survives indirect calls the decompiler chokes on.
# @runtime PyGhidra

from ghidra.app.decompiler import DecompInterface
from ghidra.program.model.data import FunctionDefinitionDataType, IntegerDataType, ParameterDefinitionImpl, VoidDataType
from ghidra.program.model.pcode import HighFunctionDBUtil
from ghidra.util.task import ConsoleTaskMonitor


class Decompiler:
    def __init__(self, prog):
        self.prog = prog
        self.mon = ConsoleTaskMonitor()
        self.dec = DecompInterface()
        self.dec.openProgram(prog)

    def _push_count(self, ins):
        n = 0
        ins = ins.getPrevious()
        while ins is not None and ins.getMnemonicString() == "PUSH":
            n += 1
            ins = ins.getPrevious()
        return n

    def _override_indirect_calls(self, f):
        """Give each indirect call a cdecl prototype with one int per preceding PUSH."""
        listing = self.prog.getListing()
        for ins in listing.getInstructions(f.getBody(), True):
            if not ins.getFlowType().isCall() or not ins.getFlowType().isComputed():
                continue
            sig = FunctionDefinitionDataType("indirect_call")
            sig.setReturnType(VoidDataType.dataType)
            sig.setCallingConvention("__cdecl")
            sig.setArguments([ParameterDefinitionImpl("a%d" % i, IntegerDataType.dataType, "") for i in range(self._push_count(ins))])
            HighFunctionDBUtil.writeOverride(f, ins.getAddress(), sig)

    def decompile(self, f, timeout):
        """Return the C of f, or a "// failed:" line.

        Some functions fail with "Unable to force merge" at a loop around an indirect call. A cdecl prototype on
        each such call fixes it. The overrides are made in a transaction that is rolled back.
        """
        res = self.dec.decompileFunction(f, timeout, self.mon)
        if res.decompileCompleted():
            return res.getDecompiledFunction().getC()
        tx = self.prog.startTransaction("indirect call prototypes")
        try:
            self._override_indirect_calls(f)
            retry = self.dec.decompileFunction(f, timeout, self.mon)
            if retry.decompileCompleted():
                return "/* NOTE: indirect calls given guessed cdecl prototypes */\n" + retry.getDecompiledFunction().getC()
        finally:
            self.prog.endTransaction(tx, False)
        return "// failed: %s\n" % res.getErrorMessage()
