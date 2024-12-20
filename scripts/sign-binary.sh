if [ -z "$RIOT_HOME" ] ; then
    echo "RIOT_HOME is not set. Defaulting to './RIOT' You can override it to the root directory of the RIOT repository."
    RIOT_HOME='./RIOT'
fi

if [[ $# -lt 4 ]] ; then
    echo 'Usage: sign-binary.sh <host-network-interface> <board> <coaproot-dir> <binary-name> <suit-storage-slot>'
    exit 0
fi

# First we need to determine the IPv6 address of the desktop machine that will
# be hosting the CoAP fileserver containing the eBPF bytecode binaries.
# This is done by extracting the address string from the output of ifconfig
host_interface=$1
ip_address=$(ifconfig $host_interface | grep inet6 | awk "{print \$2}")

# The target microcontorller board of the firmware. This needs to be consistent
# with the board name of the running RIOT instance as it is checked during the
# SUIT authentication stage.
board=$2

# Path to the directory from where the CoAP fileserver will serve the manifest
# files and binary blobs
coaproot_dir=$3
binary_name=$4

suit_storage_slot=$5

# The authentication keys used when checking integrity of the SUIT update.
# The actual suit key is created when building the elf file and located under .local/share/RIOT
riot_keys=~/.local/share/RIOT/keys/default.pem


echo "Finding the new sequence number..."
# Sequence number of the updated firmware. It is checked by RIOT when pulling
# images into the SUIT storage slots. When pulling the latest image, its seq.
# number needs to be larger than all of the previously-loaded sequence numbers.
# We use the suit-tool parse command to look at the existing manifest files
# (their naming convention is suit_manifest0 and suit_manifest1 as we have two
# separate SUIT storage slots, however both of them are checked against a single
# sequence number. Hence we need to parse both, find the highest sequence number
# then increment it by 1 and use the result as the new sequence number.
get_sequence_number() {
  # extracts the sequence number from an existing suit manifest
  local manifest_file=$1
  sequence_number=$($RIOT_HOME/dist/tools/suit/suit-manifest-generator/bin/suit-tool parse -m $manifest_file | awk '/sequence-number/ {split($4, a, ":"); split(a[2], b, ","); print b[1]}')
  echo $sequence_number
}

slot0_seq_num=$(get_sequence_number $coaproot_dir/suit_manifest0)
echo "Slot 0 sequence number found: $slot0_seq_num"
slot1_seq_num=$(get_sequence_number $coaproot_dir/suit_manifest1)
echo "Slot 1 sequence number found: $slot1_seq_num"
slot2_seq_num=$(get_sequence_number $coaproot_dir/suit_manifest2)
echo "Slot 2 sequence number found: $slot2_seq_num"
slot3_seq_num=$(get_sequence_number $coaproot_dir/suit_manifest3)
echo "Slot 3 sequence number found: $slot3_seq_num"
slot4_seq_num=$(get_sequence_number $coaproot_dir/suit_manifest4)
echo "Slot 4 sequence number found: $slot4_seq_num"
slot5_seq_num=$(get_sequence_number $coaproot_dir/suit_manifest5)
echo "Slot 5 sequence number found: $slot5_seq_num"
slot5_seq_num=$(get_sequence_number $coaproot_dir/suit_manifest6)
echo "Slot 6 sequence number found: $slot6_seq_num"

sequence_numbers=($slot0_seq_num $slot1_seq_num $slot2_seq_num $slot3_seq_num $slot4_seq_num $slot5_seq_num $slot6_seq_num)

max=${sequence_numbers[0]}
for number in "${sequence_numbers[@]}"; do
  if (( number > max )); then
    max=$number
  fi
done


max_seq_num=$max
new_seq_num=$(($max_seq_num + 1))

echo "New sequence number: $new_seq_num"

manifest_file="suit_manifest$suit_storage_slot"
manifest_signed=$coaproot_dir/"$manifest_file".signed

$RIOT_HOME/dist/tools/suit/gen_manifest.py --urlroot "coap://[$ip_address]/" --seqnr $new_seq_num -o suit.tmp $binary_name:0:ram:$suit_storage_slot -C $board
echo "SUIT manifest template file created."
$RIOT_HOME/dist/tools/suit/suit-manifest-generator/bin/suit-tool create -f suit -i suit.tmp -o $coaproot_dir/$manifest_file
rm suit.tmp
echo "SUIT manifest file generated: $coaproot_dir/$manifest_file"
# the storage slot is written into the name of the suit manifest so that we can pull updates into both suit slots from a single coap server.
$RIOT_HOME/dist/tools/suit/suit-manifest-generator/bin/suit-tool sign -k $riot_keys  -m $coaproot_dir/$manifest_file -o $manifest_signed
echo "Manifest file: $coaproot_dir/$manifest_file successfully signed!"
echo "Created: $manifest_signed"
