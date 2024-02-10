
# List of things to do:

- update the adjust-bytecode patching workflow to eliminate manual patching
  when the program includes change.

- investigate why gcoap finish call is broken
- update the bpf directory structure documentation and explanations

- test all helpers implemented so far and cross check against FC
    - f12r_vm_gcoap_resp_init;
    - f12r_vm_coap_opt_finish;
    - f12r_vm_coap_add_format;
    - f12r_vm_coap_get_pdu;



# Discussion points after the meeting

- possible reasons for rBPF performance hit:
  - rust in debug mode (fixed - rust was targeting size instead of speed)
  - memory region initialisation might be different


Findings:
- rBPF copies the memory region start and end for the context data on call to
  execute_program.

