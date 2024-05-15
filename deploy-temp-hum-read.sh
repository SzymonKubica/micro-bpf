RUST_LOG=DEBUG DOTENV=.env-nucleo ./tools/target/release/mibpf-tools --use-env deploy --bpf-source-file tools/tools/tests/test-sources/sensor-processing-update-thread.c -s 0 --binary-layout ExtendedHeader  --erase
DOTENV=.env-nucleo RUST_LOG=DEBUG ./tools/target/release/mibpf-tools --use-env execute --suit-storage-slot 0 --execution-model LongRunning --binary-layout ExtendedHeader --helper-access-verification PreFlight --helper-access-list-source ExecuteRequest --helper-indices 1 48 49 50 17 19 82 52 96 97


