RUST_LOG=DEBUG DOTENV=.env-nucleo tools --use-env deploy --bpf-source-file tools/tools/tests/test-sources/sensor-processing-update-thread.c -s 0 --binary-layout ExtendedHeader  --erase
