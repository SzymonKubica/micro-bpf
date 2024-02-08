
# List of things to do:

- investigate why the FC printf is broken
- update the adjust-bytecode patching workflow to eliminate manual patching
  when the program includes change.


# Discussion points after the meeting

- possible reasons for rBPF performance hit:
  - rust in debug mode
  - memory region initialisation might be different


Findings:
- rBPF copies the memory region start and end for the context data on call to
  execute_program.

