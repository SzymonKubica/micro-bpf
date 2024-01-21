
if [[ $# -lt 5 ]] ; then
    echo 'Usage: sign-binary.sh <host-network-interface> <sequence-number> <board> <coaproot-dir> <binary-name>'
    exit 0
fi

host_interface=$1
ip_address=$(ifconfig $host_interface | grep inet6 | awk "{print \$2}")
sequence_number=$2
board=$3
coaproot_dir=$4
binary_name=$5

echo "Creating the manifest template"
./RIOT/dist/tools/suit/gen_manifest.py --urlroot "coap://[$ip_address]/" --seqnr $sequence_number -o suit.tmp $binary_name:0:ram:0 -C $board
echo "./RIOT/dist/tools/suit/gen_manifest.py --urlroot \"coap://[$ip_address]/\" --seqnr $sequence_number -o suit.tmp $binary_name:0:ram:0 -C $board"
echo "Generating the manifest file"
./RIOT/dist/tools/suit/suit-manifest-generator/bin/suit-tool create -f suit -i suit.tmp -o $coaproot_dir/suit_manifest
echo "./RIOT/dist/tools/suit/suit-manifest-generator/bin/suit-tool create -f suit -i suit.tmp -o $coaproot_dir/suit_manifest"
echo "Signing the manifest file"
./RIOT/dist/tools/suit/suit-manifest-generator/bin/suit-tool sign -k RIOT/keys/default.pem -m $coaproot_dir/suit_manifest -o $coaproot_dir/suit_manifest.signed
echo "./RIOT/dist/tools/suit/suit-manifest-generator/bin/suit-tool sign -k RIOT/keys/default.pem -m $coaproot_dir/suit_manifest -o $coaproot_dir/suit_manifest.signed"
