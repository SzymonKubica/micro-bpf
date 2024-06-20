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

If the flashing process into stm32 fails saying 'overlapping sections' then it
is probably because the esp-idf tools have been added to the path and RIOT tries
to flash using those. Make sure RIOT uses
```
/home/szymon/Projects/ebpf-on-microcontrollers/mibpf/RIOT/dist/tools/openocd/openocd.sh
```
for flashing as opposed to some other version of openocd

### If DHT22 doesn't respond:
Adjust the constants in the DHT module so that the start LOW time is set appropriately
as below.
#define START_LOW_TIME          (20U * US_PER_MS)

