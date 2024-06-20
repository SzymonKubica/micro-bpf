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
   ```bash
   docker pull riot/riotbuild
   ```
2. Set up RIOT system base directory
   You need to ensure that the path to the base directory of RIOT OS is specified
   correctly at the top of the `Makefile` present in this directory. You can
   adjust it by editing the line 7 in the file:
   ```bash
   RIOTBASE ?= $(CURDIR)/../RIOT
   ```
   In the example above, the compilation process expects that RIOT can be accessed
   under ../RIOT relative to the current working directory.
   Please ensure that after cloning the base `mibpf` repo, you have initialised
   all git submodules using
   ```bash
   git submodule init && git submodule update
   ```
   Otherwise RIOT/ in the repo will be just an empty directory and the compilation
   won't be successful.
3. Configure `tap` networking to allow for communicating with the running RIOT
   instance using CoAP. For more detailed documentation see [here](https://doc.riot-os.org/getting-started.html#:~:text=tap0%20make%20term-,Setting%20up%20a%20tap%20network,-There%20is%20a).
   You can easily create two tap networks and a bridge using the following command
   (assuming your working directory is mibpf/mibpf-native-example):
   ```bash
   sudo ../RIOT/dist/tools/tapsetup/tapsetup
   ```
   This script should have configured your `tap` networks. You can verify that
   it was successful using `ifconfig` and looking for `tap0`, `tap1` and `tapbr0`
   network interfaces.

4. Compile and run the application binary
   Once the repository is correctly set up, you can build the application by
   executing:
   ```bash
   BUILD_IN_DOCKER=1 TOOLCHAIN=llvm WERROR=0 make flash term
   ```
5. Now you can interact with the RIOT shell, enter `help` to list all available
   commands.
6. Before you can load eBPF program into the application running in the RIOT
   native instance, you need to use `netcat` to connect the `tap` interface of
   the host OS to the UDP server of the native RIOT instance, make sure
   you have `openbsd-netcat` installed (that's the name of the Arch package, you
   might need to use an equivalent in your distro)
   Assuming the default configuration (UDP server of the native instance available
   on port 8808), you should be able to connect to it using:
   ```
   nc -6uv <riot-intance-ip-address>%tap0 8808
   ```
   Where `<riot-instance-ip-address>` should be replaced with the ip address that
   you get when using the `ifconfig` command in the RIOT shell (the one that was
   open after the build command has successfully finished). Once you have successfully
   connected to the UDP server advertising on the native instance, you should be
   able to interact with it via CoAP using the `mibpf-tools`.
7. Before being able to send programs to the device, you first need to spin up
   a fileserver over the CoAP network so that the running RIOT instance can pull
   the eBPF program images from the host desktop. This requires using the
   `aiocoap-fileserver` which can be installed using pip.
   In order to do this, you need
   to navigate to the root directory of the repo, then create a python environment
   using:
   ```bash
   python -m venv venv
   ```
   then source the environment:
   ```bash
   source venv/bin/activate
   ```
   And finally install the requirements:
   ```bash
   pip install -r requirements.txt
   ```
   After this is done, you can start the server in a separate terminal and leave
   it running so that the RIOT instance can access files in the `coaproot/`
   directory.
   ```bash
   aiocoap-fileserver coaproot
   ```

7. Use `mibpf-tools` to compile, load and execute the program on the simulated
   microcontroller (`native`)
   First, ensure that the `tools` directory contains the submodule for the
   `mibpf-tools` repo. Then, navigate there and build the tools using
   ```bash
   cargo build --release
   ```
   Once the tools are built, you can navigate to the root directory of the `mibpf`
   repo and use the following command to compile, sign, and load an eBPF program
   into the RIOT example application:
   ```bash
   ./tools/target/release/mibpf-tools deploy --bpf-source-file bpf/helper-tests/printf.c  --out-dir bpf/helper-tests/out -s 0 --riot-ipv6-addr <riot-instance-ip-address> --host-ipv6-addr <host-os-ip-address> --host-network-interface tapbr0 --board-name native
   ```
   As before the riot-ipv6-addr above is the ip address of the RIOT native instance
   and can be optained using the `ifconfig` command in the RIOT shell. The other
   ip address is the address corresponding to the host machine on the `tapbr0`
   interface. It can be found by using
   ```bash
   ifconfig tapbr0
   ```
   and getting the value of the `inet6`

   The command above will compile the eBPF program located under bpf/helper-tests/printf.c,
   then move it over to the coaproot directory, sign it using the SUIT update
   protocol tools and then send a request to the runnign RIOT instance to pull
   the compiled bytecode image.

   For convenience, I suggest aliasing the path to the executable above as `tools` or something
   so that it can be used more easily. You can learn more about the meaning
   of the arguments to the command above using
   ```bash
   ./tools/target/release/mibpf-tools deploy --help
   ```

   After the program has been successfully loaded you should see a similar output
   in the RIOT shell indicating that the image has been successfully loaded into
   one of the SUIT storage slots on the RIOT instance
   ```
   2024-03-09 18:28:47,835 # Request payload received: {"ip_addr":"fe80::cc9a:73ff:fe4a:47f6","manifest":"suit_manifest0.signed"}
   2024-03-09 18:28:47,835 # suit_worker: started.
   2024-03-09 18:28:47,835 # suit_worker: downloading "coap://[fe80::cc9a:73ff:fe4a:47f6%6]/suit_manifest0.signed"
   2024-03-09 18:28:47,839 # suit_worker: got manifest with size 294
   2024-03-09 18:28:47,839 # suit: verifying manifest signature
   2024-03-09 18:28:47,840 # suit: validated manifest version
   2024-03-09 18:28:47,840 # Retrieved sequence number: 0
   2024-03-09 18:28:47,840 # Manifest seq_no: 864, highest available: 0
   2024-03-09 18:28:47,840 # suit: validated sequence number
   2024-03-09 18:28:47,840 # Formatted component name: .ram.0
   2024-03-09 18:28:47,840 # validating vendor ID
   2024-03-09 18:28:47,840 # Comparing 547d0d74-6d3a-5a92-9662-4881afd9407b to 547d0d74-6d3a-5a92-9662-4881afd9407b from manifest
   2024-03-09 18:28:47,840 # validating vendor ID: OK
   2024-03-09 18:28:47,840 # validating class id
   2024-03-09 18:28:47,841 # Comparing bcc90984-fe7d-562b-b4c9-a24f26a3a9cd to bcc90984-fe7d-562b-b4c9-a24f26a3a9cd from manifest
   2024-03-09 18:28:47,841 # validating class id: OK
   2024-03-09 18:28:47,841 # SUIT policy check OK.
   2024-03-09 18:28:47,841 # Formatted component name: .ram.0
   2024-03-09 18:28:47,841 # Fetching firmware |█████████████████████████| 100%
   2024-03-09 18:28:47,844 # Finalizing payload store
   2024-03-09 18:28:47,844 # Verifying image digest
   2024-03-09 18:28:47,844 # Starting digest verification against image
   2024-03-09 18:28:47,844 # Install correct payload
   2024-03-09 18:28:47,844 # Stored sequence number: 864
   2024-03-09 18:28:47,844 # Verified installed payload
   2024-03-09 18:28:47,844 # Verifying image digest
   2024-03-09 18:28:47,844 # Starting digest verification against image
   2024-03-09 18:28:47,844 # Verified installed payload
   2024-03-09 18:28:47,844 # suit_worker: update successful
   ```
   Once this is done, you can send a request to the instance to start executing
   the eBPF program in the VM:
   ```bash
   ./tools/target/release/mibpf-tools  execute --riot-ipv6-addr fe80::a0d9:ebff:fed5:986b --suit-storage-slot 0 --host-network-interface tapbr0
   ```

   ```
   ```







