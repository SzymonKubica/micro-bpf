
# List of things to do:

- add subcommand for executing bytecode in a given slot on a given vm


# Discussion points after the meeting

- possible reasons for rBPF performance hit:
  - rust in debug mode
  - memory region initialisation might be different


Findings:
- rBPF copies the memory region start and end for the context data on call to
  execute_program.

