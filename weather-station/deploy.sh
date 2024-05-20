
# Long running programs
TEMP_HUM_PROGRAM_SLOT=0
SOUND_LIGHT_PROGRAM_SLOT=1
UPDATE_DISPLAY_PROGRAM_SLOT=2

# Short scripts for querying the system
QUERY_TEMPERATURE_PROGRAM_SLOT=3
QUERY_HUMIDITY_PROGRAM_SLOT=4
QUERY_SOUND_INTENSITY_PROGRAM_SLOT=5
QUERY_LIGHT_INTENSITY_PROGRAM_SLOT=6

# it takes a while to pull the program image
IMAGE_PULL_DELAY=2 # seconds

export RUST_LOG=DEBUG
export DOTENV=.env-nucleo-wifi

TOOLS=../tools/target/release/mibpf-tools

echo "Deploying the temperature/humidity collecting program..."
$TOOLS --use-env deploy --bpf-source-file src/temperature-humidity-update-thread.c \
         -s $TEMP_HUM_PROGRAM_SLOT --binary-layout ExtendedHeader  --erase

sleep $IMAGE_PULL_DELAY

$TOOLS --use-env execute --suit-storage-slot $TEMP_HUM_PROGRAM_SLOT \
  --execution-model LongRunning --binary-layout ExtendedHeader \
  --helper-access-verification PreFlight --helper-access-list-source ExecuteRequest \
  --helper-indices 1 48 49 50 17 19 82 52 96 97


echo "Deploying the light/sound intensity collecting program..."
$TOOLS --use-env deploy --bpf-source-file src/sound-light-intensity-update-thread.c \
         -s $SOUND_LIGHT_PROGRAM_SLOT --binary-layout ExtendedHeader  --erase

sleep $IMAGE_PULL_DELAY

$TOOLS --use-env execute --suit-storage-slot $SOUND_LIGHT_PROGRAM_SLOT \
  --execution-model LongRunning --binary-layout ExtendedHeader \
  --helper-access-verification PreFlight --helper-access-list-source ExecuteRequest \
  --helper-indices 1 48 49 50 17 19 82 52 96 97

sleep $IMAGE_PULL_DELAY


echo "Deploying the query temperature program..."
$TOOLS --use-env deploy --bpf-source-file src/gcoap_temperature.c \
         -s $QUERY_TEMPERATURE_PROGRAM_SLOT --binary-layout ExtendedHeader  --erase

sleep $IMAGE_PULL_DELAY

echo "Deploying the query humidity program..."
$TOOLS --use-env deploy --bpf-source-file src/gcoap_humidity.c \
         -s $QUERY_HUMIDITY_PROGRAM_SLOT --binary-layout ExtendedHeader  --erase

