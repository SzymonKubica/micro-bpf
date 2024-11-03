- [x] fix relocation resolution for raw object file binaries
- [x] separate testing endpoints from the main server
      this one is quite important, the testing endpoints should only be loaded
      into the server code if a correct build flag / feature is set. It doesn't
      make sense to have all of those endpoint intermingled with the actual useful ones.
- [ ] add conditonal compilation for the testing endpoints
- [ ] remove all comp time warnings
- [ ] refactor and document server code
- [ ] fix jit helper function call args passing
- [ ] update RIOT version
- [ ] quickstart instructions
- [ ] test running documentation
- [ ] playground website (setup and startup info)
- [ ] add lots of reminders that you need to have python .env set up.
- [ ] fix pc-relative calls (testcase pc_relative_calls fails for the raw_object_file binary layout)
      - looks like it is caused by incorrect handling of the static strings.
      - need a way to extract all .rodata.str.x sections from the elf file.
