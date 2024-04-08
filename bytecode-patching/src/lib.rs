#![no_std]
#![warn(missing_docs)]

//! Library for manipulating ELF files to allow for executing them on different
//! implementations of an eBPF VM on microcontrollers.
//!
//! The main purpose of this library it to act similar to a static linker for
//! eBPF programs to allow for executing them in constrained environments.
//! Depending on implementations details of the eBPF VM used for executing the
//! programs, different pre-processing steps of the object files are required.
//!
//! The program lifecycle is as follows:
//! - The program is compiled using `clang` and `llc` for the `bpf` target ISA.
//! - The resulting object file is processed to make it compatible with a particular
//!   version of the VM.
//!
//! Currently the following pre-processing steps are supported:
//! - Extracting the `.text` section from the ELF file and executing only that
//!   (this is used by the original version of the rbpf VM and is the simplest
//!   approach, albeit it lacks generality and the space of supported programs
//!   is limited)
//! - Applying ahead-of-time relocations used by the Femto-Container version
//!   of the VM. This allows for accessing the `.data` and `.rodata` sections
//!   of the program by using special instructions that weren't originally included
//!   in the eBPF ISA.
//! - Applying extended AOT relocations to allow for calling non-static functions
//!   inside of the eBPF programs (adds support for non-PC-relative function calls)
//!
//! The second workflow that supported by the library involves sending raw ELF
//! object files to the target microcontroller device and performing relocations
//! there once the program memory address is known. This allows for achieving
//! the best compatibility, however it comes with a performance overhead of
//! parsing the ELF file on the device each time the program is loaded.
//!
//! In order to support the second type of the relocation workflow, this library
//! supports `no_std`.
//!
//! The crate has been instrumented with debug print statements which can be
//! controlled by using an implementation of a logging library such as e.g.
//! env_logger. The reason raw print statements aren't used is to maintain the
//! compatibility with `no_std`.
extern crate alloc;
extern crate rbpf;

mod common;
mod extended_relocations;
mod femtocontainer_relocations;
mod model;
mod relocation_resolution;

// Only the below functions are exposed to the users of this library.
pub use common::debug_print_program_bytes;
pub use extended_relocations::assemble_binary;
pub use extended_relocations::assemble_binary_specifying_helpers;
pub use common::extract_section;
pub use femtocontainer_relocations::assemble_femtocontainer_binary;
pub use relocation_resolution::resolve_relocations;
