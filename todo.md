
# List of things to do:

- clean up the memory logging
- equalise the interface for passing in strings
- add command line utilities
- plot graphs

- streamline loading workflow


Current bytecode pulling workflow (each step is a single interaction)
- compile eBPF bytecode
- sign the manifest
- pull the image (requires switching to the RIOT shell)
- execute the code (requires sending a CoAP request)

# Discussion points after the meeting

- possible reasons for rBPF performance hit:
  - rust in debug mode
  - memory region initialisation might be different


Findings:
- rBPF copies the memory region start and end for the context data on call to
  execute_program.

