# Femto-Container Virtual Machine

The Femto-Container virtual machine internals are based on the eBPF and the same
ISA is implemented. This way existing work with compilers and toolchains can be
reused.

## eBPF

The eBPF virtual machine is a simple register based virtual machine. The
specification only includes the instruction set and the registers, no
additional hardware peripherals (including an interrupt controller) are
simulated.

The eBPF speification has ten 64 bit registers and supports arbitrary load and
store. All instructions operate on the register set. All jump instructions
are arbitrary, they jump to an PC relative offset contained within the
instruction.
For an overview of the instructions see the [IO Visor documentation]

The eBPF specification includes one special call instruction. This instruction
is used to call system calls from within the virtual machine. In the
Femto-Container virtual machine it is used to call implementation specific calls
provided by the host system.

## Memory

Femto-Containers shares the memory map of the host system, no address
translation is done. To still provide isolation, a memory protection system is
in place designed as denylist. Individual read and write permissions can be
attached to regions. For example, the stack region provided is configured as
read/write region.

Additional region permissions can be provided by the host system when required.
For Example when the context struct contains a pointer to further content, the
host system can provide additional permissions to the VM to read the content
(and possibly not modify it). For this the host system needs to allocate another
`f12r_mem_region_t` and supply it with the memory space and the permissions.

```C
/* Assumes there is already a femtocontainer context named femtoc */
f12r_mem_region_t additional_region;
f12r_add_region(femtoc, &additional_region,
                my_memory, sizeof(my_memory)
		FC_MEM_REGION_READ | FC_MEM_REGION_WRITE);
```

The `f12r_mem_region_t` must remain valid for the lifetime of the femtocontainer
context itself.

## Pointers

Internally the Femto-Container virtual machine is a 64 bit architecture,
identical to eBPF. This poses a problem when sharing structs containing pointers
from non 64 bit architectures: Within the virtual machine a pointer is 64 bit
wide, outside the virtual machine it can be between 16 and 64 bit wide. Sharing
structs between the host and the virtual machine with pointers in them can cause
the issue that the host and the virtual machine disagree about the size of the
struct.

To ensure that pointers always occupy at least 64 bit an union between a 64 bit
integer and the required pointer is used.

[IO Visor documentation](https://github.com/iovisor/bpf-docs/blob/master/eBPF.md)
