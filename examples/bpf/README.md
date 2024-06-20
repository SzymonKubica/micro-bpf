# eBPF Example Snippets

This directory contains example programs written in a eBPF-compatible subset
of C. The directory structure can be seen below:
```
.
├── femto-container
│   ├── ...
└── rbpf
    ├── ...
```

Two different directories are needed because currently, two different eBPF VM's
are supported in RIOT and they require slightly different compilation workflows.

The `femto-container` directory contains code which is compatible with the
Femto-Container eBPF VM included in RIOT. To allow for making eBPF helper calls,
the example eBPF programs import header files which are specific to RIOT and
the Femto-Container implementation.

The `rbpf` directory contains snippets that are compatible with rust based
rbpf VM and thus they require a different set of header files. For some
helper calls (e.g. `bpf_trace_printk`) the same header files as in the case of
Linux eBPF VM are used, however for custom os-specific helper functions that I
added, new header files are included.
