
# List of things to do:

- investigate why the FC printf is broken
- update the adjust-bytecode patching workflow to eliminate manual patching
  when the program includes change.

- investigate why gcoap finish call is broken
- update the bpf directory structure documentation and explanations

- test all helpers implemented so far and cross check against FC
    - f12r_vm_printf;
    - f12r_vm_saul_reg_find_nth;
    - f12r_vm_saul_reg_find_type;
    - f12r_vm_saul_reg_read;
    - f12r_vm_saul_reg_write;
    - f12r_vm_gcoap_resp_init;
    - f12r_vm_coap_opt_finish;
    - f12r_vm_coap_add_format;
    - f12r_vm_coap_get_pdu;
    - f12r_vm_fmt_s16_dfp;
    - f12r_vm_fmt_u32_dec;
    - f12r_vm_ztimer_now;
    - f12r_vm_ztimer_periodic_wakeup;




# Discussion points after the meeting

- possible reasons for rBPF performance hit:
  - rust in debug mode
  - memory region initialisation might be different


Findings:
- rBPF copies the memory region start and end for the context data on call to
  execute_program.

