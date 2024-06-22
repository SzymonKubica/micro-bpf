
# Long running programs
TEMP_HUM_PROGRAM_SLOT=0
SOUND_LIGHT_PROGRAM_SLOT=1
UPDATE_DISPLAY_PROGRAM_SLOT=2

# Allows for querying the data collected by the system
QUERY_PROGRAM_SLOT=3

# it takes a while to pull the program image
IMAGE_PULL_DELAY=4 # seconds

export RUST_LOG=DEBUG
export DOTENV=.env-nucleo-$1

TOOLS=../../tools/target/release/mibpf-tools

deploy_temp_hum() {
echo "Deploying the temperature/humidity collecting program..."
$TOOLS --use-env deploy --bpf-source-file src/temperature-humidity-update-thread.c \
         -s $TEMP_HUM_PROGRAM_SLOT --binary-layout ExtendedHeader  --erase

sleep $IMAGE_PULL_DELAY

$TOOLS --use-env execute --suit-storage-slot $TEMP_HUM_PROGRAM_SLOT \
  --execution-model LongRunning --binary-layout ExtendedHeader \
  --helper-access-verification PreFlight --helper-access-list-source ExecuteRequest \
  --helper-indices 1 48 49 50 17 19 82 52 96 97
}


deploy_light_sound() {
echo "Deploying the light/sound intensity collecting program..."
$TOOLS --use-env deploy --bpf-source-file src/sound-light-intensity-update-thread.c \
         -s $SOUND_LIGHT_PROGRAM_SLOT --binary-layout ExtendedHeader  --erase

sleep $IMAGE_PULL_DELAY

$TOOLS --use-env execute --suit-storage-slot $SOUND_LIGHT_PROGRAM_SLOT \
  --execution-model LongRunning --binary-layout ExtendedHeader \
  --helper-access-verification PreFlight --helper-access-list-source ExecuteRequest \
  --helper-indices 1 48 49 50 17 19 82 52 96 97
}

deploy_display_update() {
echo "Deploying the display update program..."
$TOOLS --use-env deploy --bpf-source-file src/display-update-thread.c \
         -s $UPDATE_DISPLAY_PROGRAM_SLOT --binary-layout RawObjectFile  --erase

sleep $IMAGE_PULL_DELAY

$TOOLS --use-env execute --suit-storage-slot $UPDATE_DISPLAY_PROGRAM_SLOT \
  --execution-model LongRunning --binary-layout RawObjectFile \
  --helper-access-verification PreFlight --helper-access-list-source ExecuteRequest \
  --helper-indices 1 48 49 50 17 19 82 52 96 97 131 130 132 133 134 128 129 80 81

sleep $IMAGE_PULL_DELAY
}

deploy_queries() {
echo "Deploying the query temperature program..."
$TOOLS --use-env deploy --bpf-source-file src/gcoap_temperature.c \
         -s $QUERY_PROGRAM_SLOT --binary-layout ExtendedHeader  --erase }
}
deploy_temp_hum
deploy_light_sound
deploy_display_update
