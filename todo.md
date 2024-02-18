
# List of things to do:

- update the adjust-bytecode patching workflow to eliminate manual patching
  when the program includes change.

- update the bpf directory structure documentation and explanations
- clean up the c modules. (figure out how to get them loaded in)

- allow for spawning long running vms

- test all helpers implemented so far and cross check against FC
    - f12r_vm_coap_get_pdu;

- make it possible to compile the project out of the riot tree

- fix it so that it runs on native as well.


# Done:
- investigate why gcoap finish call is broken (all fc gcoap and fmt helpers were
  broken as they passed the function arguments incorrrectly)
- gcoap_resp_init fails for some reason (ISR stack overflow). (see above)
- build a consistent setup of passing coap packets into bpf programs (done for FC)
- replicate the above for rbpf


# Discussion points after the meeting

- possible reasons for rBPF performance hit:
  - rust in debug mode (fixed - rust was targeting size instead of speed)
  - memory region initialisation might be different


Findings:
- rBPF copies the memory region start and end for the context data on call to
  execute_program.

- FC gcoap calls are broken:
// Those function calls are just broken.
// The femtocontainer VM calls the function by passing in the pointer to the
// array of registers and those expect that they will be called with a list of
// arguments.
uint32_t f12r_vm_gcoap_resp_init(f12r_t *f12r, uint32_t coap_ctx_p, uint32_t resp_code_u, uint32_t a3, uint32_t a4, uint32_t a5)
{ ... }
// Should be:
uint32_t f12r_vm_gcoap_resp_init(f12r_t *f12r, uint64_t *regs)
{ ... }


