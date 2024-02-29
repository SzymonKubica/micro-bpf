# eBPF Program Bytecode Compilation Workflow

The compilation of an eBPF program for microcontrollers involves:
- first compilation step using `clang` into the llvm bitcode
- llvm bitcode is compiled into the eBPF assembly instructions using `llc` with the **bpf** architecture option
- postprocessing step is applied depending on the implementation details
  of the target VM.

## Compilation into llvm bitcode

The first step of the compilation workflow involves emitting the llvm bitcode
using the command below:
```bash
clang -emit-llvm -c source_file.c -o
```
It creates an output file `source_file.bc` which contains the llvm bitcode and
can be inspected in a human-readable way using:

```bash
llvm-dis source_file.bc
```

## Compilation from llvm bitcode into target object file

After the bitcode is generated, it is compiled into the eBPF bytecode using:

```bash
llc -march=bpf -mcpu=v2 -filetype=obj -c source_file.bc
```

Both of the above steps are the same for the two target VMs that we are considering
(rust-based rBPF, and Femto-Containers VM). After that, the compilation workflows
diverge as the the post-processing step is tied to the specifics of what format
of the bytecode is expected by the target VM.

## Postprocessing


1. First step is to use clang to compile the source file to llvm bitcode,
   it emits the LLVM bitcode which can be inspected using

```bash
llvm-dis <bitcode_file>.bc
```

2. Then we pipe that into `lcc` to generate the eBPF object file

3. Then the manifest extraction and relocations happen



