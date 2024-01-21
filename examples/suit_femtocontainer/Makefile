# name of your application
APPLICATION = suit_femtocontainer

# If no BOARD is found in the environment, use this default:
BOARD ?= esp32-wroom-32

# This has to be the absolute path to the RIOT base directory:
RIOTBASE ?= $(CURDIR)/../../RIOT

#
# Networking
#
# Include packages that pull up and auto-init the link layer.
# NOTE: 6LoWPAN will be included if IEEE802.15.4 devices are present

# uncomment this to compile in support for a possibly available radio
#USEMODULE += netdev_default

USEMODULE += auto_init_gnrc_netif
USEMODULE += netdev_default
# Specify the mandatory networking modules for IPv6 and UDP
USEMODULE += gnrc_ipv6_router_default
USEMODULE += sock_udp
# Additional networking modules that can be dropped if not needed
USEMODULE += gnrc_icmpv6_echo

# include this for printing IP addresses
USEMODULE += shell
USEMODULE += shell_cmds_default
USEMODULE += shell_cmd_suit
USEMODULE += bpf
USEMODULE += gcoap
USEMODULE += ztimer
USEPKG += femto-container

USEMODULE += saul
USEMODULE += saul_reg
USEMODULE += saul_default



# Set this to 1 to enable code in RIOT that does safety checking
# which is not needed in a production environment but helps in the
# development process:
DEVELHELP ?= 0

# Change this to 0 show compiler invocation lines by default:
QUIET ?= 1

# SUIT update specific stuff
USEMODULE += suit
USEMODULE += suit_transport_coap
USEMODULE += suit_storage_ram

# Display a progress bar during firmware download
USEMODULE += progress_bar

# Default COAP manifest resource location when fetched through gpio trigger
CFLAGS += -DSUIT_MANIFEST_RESOURCE=\"$(SUIT_COAP_ROOT)/$(SUIT_NOTIFY_MANIFEST)\"
# 2 Storage regions in RAM with 2K in size
CFLAGS += -DCONFIG_SUIT_STORAGE_RAM_REGIONS=2 -DCONFIG_SUIT_STORAGE_RAM_SIZE=2048

# Ensure both slot bin files are always generated and linked to avoid compiling
# during the test. This ensures that "BUILD_IN_DOCKER=1 make test"
# can rely on them being present without having to trigger re-compilation.
BUILD_FILES += $(SLOT_RIOT_ELFS:%.elf=%.bin)

# The test needs the linked slot binaries without header in order to be able to
# create final binaries with specific APP_VER values. The CI RasPi test workers
# don't compile themselves and re-create signed images, thus add the required
# files here so they will be submitted along with the test jobs.
TEST_EXTRA_FILES += $(SLOT_RIOT_ELFS) $(SUIT_SEC) $(SUIT_PUB)

RIOT_VERSION = middleware22

include $(RIOTBASE)/Makefile.include

.PHONY: host-tools

host-tools:
	$(Q)env -u CC -u CFLAGS $(MAKE) -C $(RIOTTOOLS)


# allow to use large blocks to utilize large MTUs (802.15.4g, Ethernet, WiFi)
LARGE_BLOCKS ?= 1
ifeq (1, $(LARGE_BLOCKS))
  CFLAGS += -DCONFIG_SUIT_COAP_BLOCKSIZE=COAP_BLOCKSIZE_1024
endif

