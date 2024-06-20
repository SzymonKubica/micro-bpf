# This script calculates the lines of code in each of the parts of the system.
# This is needed to document implementation in the project report.

# mibpf-server + ffi
echo "mibpf-server rust:"
server_rust_lines=$(find mibpf-server/src | grep -F .rs | xargs wc -l | grep total | awk '{print $1}')
echo $server_rust_lines
echo "mibpf-server ffi:"
server_ffi_lines=$(find mibpf-server/src | grep -F .c | xargs wc -l | grep total | awk '{print $1}')
echo $server_ffi_lines
echo "rbpf-for-microcontrollers no ARMv7 jit:"
rbpf_no_jit_lines=$(find rbpf-for-microcontrollers/src | grep -v thumb | grep -F .rs | xargs wc -l | grep total | awk '{print $1}')
echo $rbpf_no_jit_lines
echo "rbpf-for-microcontrollers ARMv7 jit:"
rbpf_jit_lines=$(find rbpf-for-microcontrollers/src | grep thumb | grep -F .rs | xargs wc -l | grep total | awk '{print $1}')
echo $rbpf_jit_lines
echo "rbpf-for-microcontrollers total:"
rbpf_total=$(find rbpf-for-microcontrollers/src | grep -F .rs | xargs wc -l | grep total | awk '{print $1}')
echo $rbpf_total

echo "mibpf tools total:"
mibpf_tools=$(find tools/*/src | grep -F .rs | xargs wc -l | grep total | awk '{print $1}')
echo $mibpf_tools
echo "mibpf tools tests:"
mibpf_tools_tests=$(find tools/*/tests | grep -E '\.rs|\.c' | xargs wc -l | grep total | awk '{print $1}')
echo $mibpf_tools_tests

echo "Total lines of Rust:"
echo $(($server_rust_lines + $rbpf_no_jit_lines + $rbpf_jit_lines + $mibpf_tools))

echo "Total lines of C:"
echo $server_ffi_lines


echo "Total test files:"
echo $mibpf_tools_tests




