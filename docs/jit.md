# JIT Motivation, Planning and Implementation Notes

**Ingredients needed:**
- memory and address translation
- register allocation
- instruction translation

Key difficulties:
- calling helper functions requires matching the callling convention used
  by rust on the target architecture


Components of the jit compiler:

- [x] infrastructure for casting a pointer to the bytecode as a function pointer
      and calling into it.
- [ ] structs modelling the registers
- [ ] structs modelling instructions
- [ ] some mechanism for allocating memory for the jit-compiled program


Calling into the jit-compiled bytecode:
Given that the CPU used by the target microcontroller (ARM Cortex M4) uses
the thumbv7em architecture, when calling into the jit-compiled code, we need
to set the least significant bit in the function pointer to the bytecode.
The reason we need this is that the CPU needs to know that we are calling into
a function that is supposed to execute in Thumb to ensure that interworking
works correctly.

Interworking is the seamless interoperability between the regular 32 bit ARM
instructions and the simplified 16 bit Thumb instructions.

The reason we normally don't have to do this when calling functions is that the
compilation toolchain handles this for us. However when we are doing this work
manually (explicitly casting a pointer to the compiled bytes as a function pointer),
we need to set that LSB indicator ourselves.

Function workflow:
- push LR onto the stack,
- push all callee-saved registers
- do stuff
- pop all callee-saved registers
- pop PC
- emit return via `bx lr`

The reason we push LR and then pop PC is the following:
- when the function is called, the LR (link-register) will contain the previous
  value of the program counter from where the function was called (it will be
  stored there because we call the function using the BL (branch-with-link) instruction)
  So we take the LR register and push it onto the stack.
- once we are done with our function business, we need to restore the PC to the
  previous value (to point to the instruction that called us) assuming that the
  stack was handled correctly and the previously pushed LR value is at the top
  of the stack, we can pop it into PC and end up with a consistent state where
  the execution resumes from the previous value of the PC when we have made our call.
