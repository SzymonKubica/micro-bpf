# Performing relocations on the target device

Existing solutions perform a bytecode patching step before sending the binary
to the target device. This ensures that the program executing int the VM can
access function calls and `.rodata` sections correctly.

The problem with this approach is that it requires introducing new instructions
to the eBPF instruction set (LDDWD AND LDDWR) which instruct the VM interpreter
that a load-double-word instruction (LDDW) is supposed to load data not directly
from the address specififed by the instruction immediate operand, rather it should
use the immediate operand as an offset from the start of the `.data` or `.rodata`
sections.

This is somewhat hacky and requires changes to the VM interpreter, thereby
coupling the interpreter behaviour with the bytecode patching procedure.

An alternative solution would be to send the raw object file directly to the
binary and perform relocations there. The limiations of this approach are that the
object files include a lot of debug information and therefore the binaries might
not fit int the constrained RAM of the target device. A possible solution would
be to perform a pre-processing step which only stips off the `.text`, `.data`,
`.rodata` and relocations sections and then sends the resulting binary to the
device where the corresponding relocations can be applied.

The implementation of this solution should be evaluated against the following metrics:
- RAM requirement of the relocation process
- load time overhead of parsing the object file on the device
- the classes of programs that can be supported using the solution (e.g. can we
read from the .rodata sections or can we function pointers in the `.data` section)

**Note**: turns out that a viable approach for reducing object file size is to
use the `strip` command that allows for getting rid of the debug information.
It also allows for removing the `.BTF` sections that we don't need.

In the example of the simple CoAP response formatting application
`bpf/helper-tests/out/gcoap.o`, it was possible to reduce the object file size
9704B to just under 2kB.


Steps towards implementing:
- first: preprocess the object files by removing debug symbols and `.BTF`, `.BTF.ext`
  sections.
- add a new endpoint which loads in the object and performs proper relocations
  before executing the VM on it.


