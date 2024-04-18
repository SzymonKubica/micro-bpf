# JIT Motivation, Planning and Implementation Notes

**Ingredients needed:**
- memory and address translation
- register allocation
- instruction translation

Key difficulties:
- calling helper functions requires matching the callling convention used
  by rust on the target architecture


Components of the jit compiler:

- [ ] structs modelling the registers
- [ ] structs modelling instructions
- [ ] some mechanism for allocating memory for the jit-compiled program

