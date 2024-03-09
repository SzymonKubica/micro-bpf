# Example Application for executing eBPF VMs running on a native target.

This directory contains an example application allowing for testing the compile
-load-execute workflow of eBPF programs on microcontrollers running RIOT.
It is compatible with the `native` RIOT target which runs an instance of the OS
directly on the host desktop machine.

## Quickstart guide

1. Install dependencies for compilation
   Because of rust-llvm compatibility issues, the preferred approach for building
   this example is to use the BUILD_IN_DOCKER functionality provided by RIOT, in
   order to use this, you need to have `docker` installed and then pull the latest
   version of the RIOT build image required for building the project
   ```
   docker pull riot/riotbuild
   ```
2. Set up RIOT system base directory
   You need to ensure that the path to the base directory of RIOT OS is specified
   correctly at the top of the `Makefile` present in this directory. You can
   adjust it by editing the line 7 in the file:
   ```
   RIOTBASE ?= $(CURDIR)/../RIOT
   ```
   In the example above, the compilation process expects that RIOT can be accessed
   under ../RIOT relative to the current working directory.
   Please ensure that after cloning the base `mibpf` repo, you have initialised
   all git submodules using
   ```
   git submodule init
   ```
   Otherwise RIOT/ in the repo will be just an empty directory and the compilation
   won't be successful.
3. Compile and run the application binary
4. Use `mibpf-tools` to compile, load and execute the program on the simulated
   microcontroller (`native`)


The system image needs to be compiled in docker because of compatibility issues
with `c2rust` that RIOT uses for compiling applications containing rust.




