if  [ $# -lt 2 ]; then
    echo "Usage: $0 <bpf-bin-file> <coaproot-dir>"
    exit 1
fi

bin_file=$1
coaproot_dir=$2
make -C examples/suit_femtocontainer/bpf clean
echo "Compiling the eBPF binary."
make -C examples/suit_femtocontainer/bpf
echo "Copying the binary to the coap root directory"
cp examples/suit_femtocontainer/bpf/$bin_file $coaproot_dir
