
# List of things to do:

- update the bpf directory structure documentation and explanations
- test all helpers implemented so far and cross check against FC
    - f12r_vm_coap_get_pdu;

- fix the benchmark endpoint
- clean up unused gcoap endpoints
- update tools to allow for switching between the patching script backend.
- build testsuite on native
- clean up the logging situation with rBPF

# Done:
- investigate why gcoap finish call is broken (all fc gcoap and fmt helpers were
  broken as they passed the function arguments incorrrectly)
- gcoap_resp_init fails for some reason (ISR stack overflow). (see above)
- build a consistent setup of passing coap packets into bpf programs (done for FC)
- replicate the above for rbpf
- make it possible to compile the project out of the riot tree
- allow for spawning long running vms
- clean up the c modules. (figure out how to get them loaded in)
- clean up the main codebase
- finish fixing compilation issues
- write up the binary blob creation workflow
- refactor bytecode patching script
- fix it so that it runs on native as well.
- modify rBPF to allow for switching between the two bytecode shape paradigms:
  - just the text section
  - full program with header, data, rodata, text + possibly aot compiled
- add support for global storage
- make it so that most of the dev is done on native


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


