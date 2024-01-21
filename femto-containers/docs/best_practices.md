# Femto-Container best practices

This is a rough collection of best practices and gotchas when dealing with
Femto-Containers.

## Sharing Structs

When sharing data structs between Femto-Containers, make sure to always use
fixed width data types (`uint8_t` to `uint64_t` and their signed counterparts).
The architecture of the host system can be different enough that the width of a
word is different than what is used by the compiler inside the virtual machine.
The size of an `int` data type can thus be different on the host system and the
virtual machine.

## Sharing Memory

Memory can be shared directly with the virtual machine as long as proper
permissions are attached to the memory regions. In fact this is the fastest way
to get data inside the container.
