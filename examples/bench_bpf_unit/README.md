# Bench BPF unit test example

This benchmark application benchmarks a number of representative instructions on
different BPF interpreters for RIOT. The output contains the time it takes for
each instruction to run. The benchmark application can be compiled and executed
on any RIOT supported board.

Toggling the interpreter is done via the `FEMTO` and `BPF_COQ` variables in the
Makefile.

## Compiling and Running the example

### Running directly on Linux

Compiling this example can be done via

```Console
koen@zometeen examples/bench_bpf_unit $ make all
Building application "tests_bench_bpf_unit" for "native" with MCU "native".

rm -Rf RIOT/build/pkg/femto-container
mkdir -p $(dirname RIOT/build/pkg/femto-container)
cp -a RIOT/../femto-containers RIOT/build/pkg/femto-container
touch RIOT/build/pkg/femto-container/.prepared
"make" -C RIOT/pkg/femto-container/
"make" -C RIOT/build/pkg/femto-container/src -f RIOT/pkg/femto-container/Makefile.femto MODULE=femto-container
"make" -C RIOT/boards/common/init
"make" -C RIOT/boards/native
"make" -C RIOT/boards/native/drivers
"make" -C RIOT/core
"make" -C RIOT/core/lib
"make" -C RIOT/cpu/native
"make" -C RIOT/cpu/native/periph
"make" -C RIOT/cpu/native/stdio_native
"make" -C RIOT/drivers
"make" -C RIOT/drivers/periph_common
"make" -C RIOT/drivers/saul
"make" -C RIOT/drivers/saul/init_devs
"make" -C RIOT/sys
"make" -C RIOT/sys/auto_init
"make" -C RIOT/sys/bpf
"make" -C RIOT/sys/btree
"make" -C RIOT/sys/div
"make" -C RIOT/sys/embunit
"make" -C RIOT/sys/fmt
"make" -C RIOT/sys/frac
"make" -C RIOT/sys/memarray
"make" -C RIOT/sys/phydat
"make" -C RIOT/sys/saul_reg
"make" -C RIOT/sys/test_utils/interactive_sync
"make" -C RIOT/sys/ztimer
   text    data     bss     dec     hex filename
  37854    1808   64496  104158   196de examples/bench_bpf_unit/bin/native/tests_bench_bpf_unit.elf
```

In this case the example is compiled for RIOT native, so that it can be run
directly as application on a Linux installation. Running the example can be done
with:

```Console
koen@zometeen examples/bench_bpf_unit $ make term
examples/bench_bpf_unit/bin/native/tests_bench_bpf_unit.elf /dev/ttyACM0 
RIOT native interrupts/signals initialized.
RIOT native board initialized.
RIOT native hardware initialization complete.

Help: Press s to start test, r to print it is ready
s
START
main(): This is RIOT! (Version: UNKNOWN (builddir: RIOT))
idx,test,duration,code,usperinst,instrpersec
0,ALU neg64,0.001000,-1,0.000500,2000000.000000
1,ALU Add,0.004000,0,0.002000,500000.000000
2,ALU Add imm,0.005000,0,0.002500,400000.000000
3,ALU mul imm,0.005000,0,0.002500,400000.000000
4,ALU rsh imm,0.005000,0,0.002500,400000.000000
5,ALU div imm,0.009000,0,0.004500,222222.222222
6,MEM ldxdw,0.010000,0,0.005000,200000.000000
7,MEM stdw,0.009000,0,0.004500,222222.222222
8,MEM stxdw,0.009000,0,0.004500,222222.222222
9,Branch always,0.005000,0,0.002500,400000.000000
10,Branch eq (jump),0.007000,0,0.003500,285714.285714
11,Branch eq (cont),0.004000,0,0.002000,500000.000000
^C
native: exiting
```

As the instance is running on a unconstrained Linux computer the performance
benchmarks will be multiple times faster than on an embedded target.

### Running on a target board

The `BOARD` variable can be supplied with the `make` command to adjust the
target of the compilation.
To compile for an embedded target supported by RIOT, for example the
[nRF52840 development kit](https://www.nordicsemi.com/Products/Development-hardware/nRF52840-DK),
the following can be run:

```
koen@zometeen ~/dev/middleware2022-femtocontainers/examples/bench_bpf_unit $ make all flash BOARD=nrf52840dk
Building application "tests_bench_bpf_unit" for "nrf52840dk" with MCU "nrf52".

"make" -C RIOT/pkg/femto-container/
"make" -C RIOT/build/pkg/femto-container/src -f RIOT/pkg/femto-container/Makefile.femto MODULE=femto-container
"make" -C RIOT/boards/common/init
"make" -C RIOT/boards/nrf52840dk
"make" -C RIOT/boards/common/nrf52xxxdk
"make" -C RIOT/core
"make" -C RIOT/core/lib
"make" -C RIOT/cpu/nrf52
"make" -C RIOT/cpu/cortexm_common
"make" -C RIOT/cpu/cortexm_common/periph
"make" -C RIOT/cpu/nrf52/periph
"make" -C RIOT/cpu/nrf52/vectors
"make" -C RIOT/cpu/nrf5x_common
"make" -C RIOT/cpu/nrf5x_common/periph
"make" -C RIOT/drivers
"make" -C RIOT/drivers/periph_common
"make" -C RIOT/drivers/saul
"make" -C RIOT/drivers/saul/init_devs
"make" -C RIOT/sys
"make" -C RIOT/sys/auto_init
"make" -C RIOT/sys/bpf
"make" -C RIOT/sys/btree
"make" -C RIOT/sys/div
"make" -C RIOT/sys/embunit
"make" -C RIOT/sys/fmt
"make" -C RIOT/sys/frac
"make" -C RIOT/sys/isrpipe
"make" -C RIOT/sys/malloc_thread_safe
"make" -C RIOT/sys/memarray
"make" -C RIOT/sys/newlib_syscalls_default
"make" -C RIOT/sys/phydat
"make" -C RIOT/sys/saul_reg
"make" -C RIOT/sys/stdio_uart
"make" -C RIOT/sys/test_utils/interactive_sync
"make" -C RIOT/sys/tsrb
"make" -C RIOT/sys/ztimer
   text    data     bss     dec     hex filename
  28144     532   19220   47896    bb18 examples/bench_bpf_unit/bin/nrf52840dk/tests_bench_bpf_unit.elf
RIOT/dist/tools/jlink/jlink.sh flash examples/bench_bpf_unit/bin/nrf52840dk/tests_bench_bpf_unit.bin
### Flashing Target ###
### Flashing at base address 0x0 with offset 0 ###
SEGGER J-Link Commander V7.58 (Compiled Nov  4 2021 16:27:58)
DLL version V7.58, compiled Nov  4 2021 16:27:42

J-Link Commander will now exit on Error

J-Link Command File read successfully.
Processing script file...

J-Link connection not established yet but required for command.
Connecting to J-Link via USB...O.K.
Firmware: J-Link OB-SAM3U128-V2-NordicSemi compiled Feb  2 2021 16:47:20
Hardware version: V1.00
S/N: 683806234
License(s): RDI, FlashBP, FlashDL, JFlash, GDB
VTref=3.300V
Target connection not established yet but required for command.
Device "NRF52" selected.


Connecting to target via SWD
InitTarget() start
InitTarget() end
Found SW-DP with ID 0x2BA01477
DPIDR: 0x2BA01477
Scanning AP map to find all available APs
AP[2]: Stopped AP scan as end of AP map has been reached
AP[0]: AHB-AP (IDR: 0x24770011)
AP[1]: JTAG-AP (IDR: 0x02880000)
Iterating through AP map to find AHB-AP to use
AP[0]: Core found
AP[0]: AHB-AP ROM base: 0xE00FF000
CPUID register: 0x410FC241. Implementer code: 0x41 (ARM)
Found Cortex-M4 r0p1, Little endian.
FPUnit: 6 code (BP) slots and 2 literal slots
CoreSight components:
ROMTbl[0] @ E00FF000
[0][0]: E000E000 CID B105E00D PID 000BB00C SCS-M7
[0][1]: E0001000 CID B105E00D PID 003BB002 DWT
[0][2]: E0002000 CID B105E00D PID 002BB003 FPB
[0][3]: E0000000 CID B105E00D PID 003BB001 ITM
[0][4]: E0040000 CID B105900D PID 000BB9A1 TPIU
[0][5]: E0041000 CID B105900D PID 000BB925 ETM
Cortex-M4 identified.
Halting CPU for downloading file.
Downloading file [examples/bench_bpf_unit/bin/nrf52840dk/tests_bench_bpf_unit.bin]...
Comparing flash   [100%] Done.
Erasing flash     [100%] Done.
Programming flash [100%] Done.
J-Link: Flash download: Bank 0 @ 0x00000000: 1 range affected (32768 bytes)
J-Link: Flash download: Total: 1.265s (Prepare: 0.091s, Compare: 0.005s, Erase: 0.665s, Program & Verify: 0.448s, Restore: 0.055s)
J-Link: Flash download: Program & Verify speed: 71 KB/s
O.K.

Reset delay: 0 ms
Reset type NORMAL: Resets core & peripherals via SYSRESETREQ & VECTRESET bit.
Reset: Halt core after reset via DEMCR.VC_CORERESET.
Reset: Reset device via AIRCR.SYSRESETREQ.



Script processing completed.
```

This compiles and flashes the firmware on the connected board. Note that this
requires a toolchain (arm-none-eabi-gcc) and flashing tooling to be installed.
See the [RIOT help for flashing](https://api.riot-os.org/flashing.html)
around this for more information and help with different supported boards.

When the board is flashed, the output from the test can be viewed via the serial
connection via:

```Console
koen@zometeen examples/bench_bpf_unit $ make term BOARD=nrf52840dk
RIOT/dist/tools/pyterm/pyterm -p "/dev/ttyACM0" -b "115200"
Twisted not available, please install it if you want to use pyterm's JSON capabilities
2022-10-21 15:39:49,399 # Connect to serial port /dev/ttyACM0
RIOT/dist/tools/pyterm/pyterm:289: DeprecationWarning: setDaemon() is deprecated, set the daemon attribute instead
  receiver_thread.setDaemon(1)
Welcome to pyterm!
Type '/exit' to exit.
s
2022-10-21 15:39:54,927 # START
2022-10-21 15:39:54,936 # main(): This is RIOT! (Version: UNKNOWN (builddir: RIOT))
2022-10-21 15:39:54,940 # idx,test,duration,code,usperinst,instrpersec
2022-10-21 15:39:54,945 # 0,ALU neg64,0.016000,-1,0.008000,125000.000000
2022-10-21 15:39:54,950 # 1,ALU Add,0.892000,0,0.446000,2242.152588
2022-10-21 15:39:54,956 # 2,ALU Add imm,1.048000,0,0.524000,1908.396973
2022-10-21 15:39:54,962 # 3,ALU mul imm,1.142000,0,0.571000,1751.313477
2022-10-21 15:39:54,968 # 4,ALU rsh imm,1.205000,0,0.602500,1659.750977
2022-10-21 15:39:54,977 # 5,ALU div imm,4.111000,0,2.055500,486.499634
2022-10-21 15:39:54,984 # 6,MEM ldxdw,2.298000,0,1.149000,870.322021
2022-10-21 15:39:54,990 # 7,MEM stdw,2.142000,0,1.071000,933.706787
2022-10-21 15:39:54,997 # 8,MEM stxdw,2.267000,0,1.133500,882.223206
2022-10-21 15:39:55,003 # 9,Branch always,0.891000,0,0.445500,2244.668945
2022-10-21 15:39:55,009 # 10,Branch eq (jump),1.454000,0,0.727000,1375.515869
2022-10-21 15:39:55,016 # 11,Branch eq (cont),1.017000,0,0.508500,1966.568359
```

Here there is a notable performance overhead per instruction. The output is a
CSV-derived format with the following columns:

* `idx`: test index
* `test`: eBPF instruction tested
* `duration`: Full duration in milliseconds of this test case.
* `code`: Return code of the instruction, was it successful?
* `usperinst`: Microseconds per individual instruction.
* `instrpersec`: Number of instructions of this type that can be executed per
  second.

Depending on the board used and the performance of the microcontroller, these
numbers may vary.



