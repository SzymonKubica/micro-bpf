[![Compile and test](https://github.com/future-proof-iot/Femto-Container/actions/workflows/tests.yml/badge.svg)](https://github.com/future-proof-iot/Femto-Container/actions/workflows/tests.yml)

# Femto-Containers

Femto-Containers is a minimal lightweight virtual machine environment for
embedded devices. The virtual machine ISA is adapted from Linux [eBPF].
Femto-Containers is pure C and makes use of some GCC extensions for efficiency.
There is some auxiliary Python tooling to convert compiled Femto-Container
applications into efficient representations.

- [Installation](#installation)
- [Usage](#usage)
- [Development](#development)
- [Further Reading and References](#further-reading--references)

# Installation

Femto-Containers is easy to integrate into your existing project. Simply include
all sources from the `src` directory into your compilation infrastructure and
include the `include` directory in your relevant files.

# Usage



# Development

Development of Femto-Containers is still in early stage. Documentation might be
lacking and the API is not yet stabilized. This will get better in the near
future.

# Further Reading & References

This [preprint](https://arxiv.org/pdf/2106.12553.pdf) describes Femto-Containers in details.
This [paper](https://arxiv.org/pdf/2011.12047.pdf) describes the rBPF interpreter and the porting of the eBPF instruction set to various microcontrollers.

[eBPF]: https://ebpf.io/

