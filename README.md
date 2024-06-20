<picture>
  <img src="examples/docs/logo-square-shadow-dark.png" width="300">
</picture>

Using eBPF for microcontroller compartmentalization.

* [Description](#description)
* [Project directory structure](#project-directory-structure)
* [Programming model](#programming-model)
* [Deployment workflow](#deployment-workflow)
* [Getting started](#getting-started)


## Description

This repository contains an end-to-end system for deploying and executing
eBPF programs on embedded devices.

`micro-bpf` consists of an eBPF virtual machine (VM), a just-in-time (JIT)
compiler targeting the ARMv7-eM architecture, server infrastructure compatible
with RIOT and a set of tools allowing to send eBPF program logic to
microcontroller devices and execute it there.

eBPF (extended Berkeley Packet Filter) is an instruction set architecture (ISA)
used in the Linux kernel to allow for executing custom user-defined code inside
of the kernel in a safe way. Although originally eBPF was intended to run in the
kernel, it can be used as a general-purposed fault isolation technology.

Because of its simplicity and support for program verification, eBPF can be
used in the context of embedded devices to provide a container-like environment
for sandboxed execution similar to docker.

The general idea is to compile programs written in a constrained subset of C
(or any other compatible front end) into eBPF bytecode and then send the
bytecode instructions to the target embedded device where they can later be
executed in an isolated VM envrionment. This allows for isolating the
underlying system from the code running in the VM. Additionally, being able to
load arbitrary programs means that the business logic deployed on the target
devices can be updated over-the-air without the need to reboot/reflash the
microcontroller.

## Project directory structure

## System architecture

## Programming model

## Deployment workflow

## Getting started

