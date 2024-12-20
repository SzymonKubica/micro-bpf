
# Long running programs
SENSOR_READ_PROGRAM_SLOT=0
#UPDATE_DISPLAY_PROGRAM_SLOT=1

# Short scripts for querying the system
QUERY_TEMPERATURE_PROGRAM_SLOT=1
QUERY_HUMIDITY_PROGRAM_SLOT=2


export RUST_LOG=INFO
export DOTENV=.env-nucleo-wifi

TOOLS=../tools/target/release/mibpf-tools

$TOOLS --use-env execute --suit-storage-slot $QUERY_TEMPERATURE_PROGRAM_SLOT \
  --execution-model WithAccessToCoapPacket --binary-layout ExtendedHeader \
  --helper-access-verification AheadOfTime --helper-access-list-source ExecuteRequest \
  --helper-indices 80 64 75 65 66 1 2 19

sleep 1
$TOOLS --use-env execute --suit-storage-slot $QUERY_HUMIDITY_PROGRAM_SLOT \
  --execution-model WithAccessToCoapPacket --binary-layout ExtendedHeader \
  --helper-access-verification AheadOfTime --helper-access-list-source ExecuteRequest \
  --helper-indices 80 64 75 65 66 1 2 19

