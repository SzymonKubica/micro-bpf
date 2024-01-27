This is a placeholder readme. Need to commit something before I add the
git submodule.

# Generating the keys:
./RIOT/dist/tools/suit/gen_key.py default.pem

# The actual key that is used is stored under
~/.local/share/RIOT/keys

# Document esp32 toolchain setup:

 ./RIOT/dist/tools/esptools/install.sh all

 get_idf

# If it fails to flash on stm32
Ensure that you have not esp32 toolchain in the path which will cause some flashing
issues.

# Troubleshooting:

If suit fetch fails and the message is 'hdr invalid' it means that cbor is
missing from python env and that the RIOT image was compiled without the
proper python env initialised.
