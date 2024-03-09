# name of your application
APPLICATION = mibpf_server

# If no BOARD is found in the environment, use this default:
BOARD ?= native

# This has to be the absolute path to the RIOT base directory:
RIOTBASE ?= $(CURDIR)/../RIOT

# Basic networking, and gcoap
USEMODULE += gcoap
USEMODULE += netdev_default
USEMODULE += auto_init_gnrc_netif
USEMODULE += gnrc_ipv6_default
USEMODULE += gnrc_icmpv6_echo
USEMODULE += gnrc_udp
USEMODULE += core_msg

USEMODULE += ztimer
USEMODULE += ztimer_usec
USEMODULE += ztimer_msec
USEMODULE += ztimer_sec

USEMODULE += periph_gpio

USEMODULE += shell
USEMODULE += shell_cmds_default
USEMODULE += shell_cmd_suit
USEMODULE += shell_commands
USEMODULE += ps

USEMODULE += suit
USEMODULE += suit_transport_coap
USEMODULE += suit_storage_ram

USEMODULE += saul
USEMODULE += saul_reg
USEMODULE += saul_default
#USEMODULE += dht

USEMODULE += fmt

USEMODULE += progress_bar

USEMODULE += vfs
USEMODULE += constfs

USEPKG += femto-container

# Include the custom module with FFI functions that are used in the rust
# project
USEMODULE += ffi
DIRS += $(CURDIR)/src/ffi
INCLUDES += -I$(CURDIR)/src/ffi/include


# Comment this out to disable code in RIOT that does safety checking
# which is not needed in a production environment but helps in the
# development process:
DEVELHELP ?= 1

# Change this to 0 show compiler invocation lines by default:
QUIET ?= 1

# Add 3k extra stack: The Rust examples take more of it than gcoap expects,
# presumably because the example use the standard library's sting formatting
# instead of one of the more optimized formatters.
CFLAGS += -DGCOAP_STACK_SIZE='(THREAD_STACKSIZE_DEFAULT+DEBUG_EXTRA_STACKSIZE+sizeof(coap_pkt_t)+ 16384)'

# Default COAP manifest resource location when fetched through gpio trigger
CFLAGS += -DSUIT_MANIFEST_RESOURCE=\"$(SUIT_COAP_ROOT)/$(SUIT_NOTIFY_MANIFEST)\"
# 2 Storage regions in RAM with 2K in size
CFLAGS += -DCONFIG_SUIT_STORAGE_RAM_REGIONS=2 -DCONFIG_SUIT_STORAGE_RAM_SIZE=2048



# Need more stack space for shell printf and threading
# We need more room for two CoAP stacks
CFLAGS += '-DTHREAD_STACKSIZE_MAIN=(THREAD_STACKSIZE_DEFAULT + 20480)'

# The name of crate (as per Cargo.toml package name, but with '-' replaced with '_')
APPLICATION_RUST_MODULE = mibpf_server
BASELIBS += $(APPLICATION_RUST_MODULE).module

FEATURES_REQUIRED += rust_target

CARGO_CHANNEL ?= stable

# Currently unknown, something related to the LED_PORT definition that doesn't
# pass C2Rust's transpilation
BOARD_BLACKLIST := ek-lm4f120xl

include $(RIOTBASE)/Makefile.include
