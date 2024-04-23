# Implementing ARMv7e-M JIT compiler for eBPF

## Motivation
- timing requierements performance critical applications (e.g. interfacing with a DHT22 sensor)

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

It seems like the 32-bit versions of the instructions (Thumb2) aren't supported.
TODO: investigate


Endianness discussion
: need to write double-word instructions correctly

Interesting observations:
the compiler actually emits add with negative immediates to subtract

Conditional branch only allows for jumping over an even number of instructions
so we need to ensure that conditional bodies are of even length

Solution: emit a no-op instruction to make the offset even

Why does the compiler emit logical shifts left followed immediately by asr?
something like this:
```
 0:   b7 01 00 00 64 00 00 00         mov %r1,100
 8:   6b 1a fe ff 00 00 00 00         stxh [%r10-2],%r1
10:   69 a1 fe ff 00 00 00 00         ldxh %r1,[%r10-2]
18:   67 01 00 00 30 00 00 00         lsh %r1,48
20:   c7 01 00 00 30 00 00 00         arsh %r1,48
28:   b7 00 00 00 7b 00 00 00         mov %r0,123
30:   65 01 01 00 9c ff ff ff         jsgt %r1,-100,1
38:   b7 00 00 00 14 00 00 00         mov %r0,20
```
That's very interesting

Talk about problems with mod not being directly supported.

Need to figure out the calling convention that rust uses.

Talk about how we need to preserve the contents of LR when calling functions
