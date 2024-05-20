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

USEMODULE += ztimer
USEMODULE += ztimer_usec
USEMODULE += ztimer_msec
USEMODULE += ztimer_sec

USEMODULE += periph_gpio
USEMODULE += periph_adc

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
USEMODULE += dht

USEMODULE += fmt

USEMODULE += progress_bar

USEMODULE += vfs
USEMODULE += constfs


# Required to use the bpf global storage.
USEMODULE += bpf
USEMODULE += btree
USEMODULE += memarray
USEPKG += femto-container

DISABLE_MODULE += mpu_stack_guard
FEATURES_BLACKLIST += cortexm_mpu

# Include the custom module with FFI functions that are used in the rust
# project
USEMODULE += ffi
DIRS += $(CURDIR)/src/ffi


# Comment this out to disable code in RIOT that does safety checking
# which is not needed in a production environment but helps in the
# development process:
DEVELHELP ?= 1

# Change this to 0 show compiler invocation lines by default:
QUIET ?= 1

# Add 3k extra stack: The Rust examples take more of it than gcoap expects,
# presumably because the example use the standard library's sting formatting
# instead of one of the more optimized formatters.
CFLAGS += -DGCOAP_STACK_SIZE='(THREAD_STACKSIZE_DEFAULT+DEBUG_EXTRA_STACKSIZE+sizeof(coap_pkt_t)+ 8192+4096)'

# Default COAP manifest resource location when fetched through gpio trigger
CFLAGS += -DSUIT_MANIFEST_RESOURCE=\"$(SUIT_COAP_ROOT)/$(SUIT_NOTIFY_MANIFEST)\"

# Config for the SUIT RAM storage
CFLAGS += -DCONFIG_SUIT_STORAGE_RAM_REGIONS=6 -DCONFIG_SUIT_STORAGE_RAM_SIZE=3072

# config for the wifi board
USEMODULE += atwinc15x0
CFLAGS += -DATWINC15X0_PARAM_SSN_PIN=GPIO_PIN\(1,6\)
CFLAGS += -DATWINC15X0_PARAM_RESET_PIN=GPIO_PIN\(1,4\)
CFLAGS += -DATWINC15X0_PARAM_IRQ_PIN=GPIO_PIN\(0,8\)

USEMODULE += hd44780

# Pinout config for the HD44780 LCD display
CFLAGS += -DHD44780_PARAM_PIN_RS=GPIO_PIN\(0,10\)
CFLAGS += -DHD44780_PARAM_PIN_RW=GPIO_UNDEF
CFLAGS += -DHD44780_PARAM_PIN_ENABLE=GPIO_PIN\(0,9\)
CFLAGS += -DHD44780_PARAM_PINS_DATA=\{GPIO_PIN\(1,13\),GPIO_PIN\(1,14\),GPIO_PIN\(1,15\),GPIO_PIN\(1,1\),GPIO_UNDEF,GPIO_UNDEF,GPIO_UNDEF,GPIO_UNDEF\}



# Need more stack space for shell printf and threading
CFLAGS += '-DTHREAD_STACKSIZE_MAIN=(THREAD_STACKSIZE_DEFAULT + 4096)'

# The name of crate (as per Cargo.toml package name, but with '-' replaced with '_')
APPLICATION_RUST_MODULE = mibpf_server
BASELIBS += $(APPLICATION_RUST_MODULE).module

FEATURES_REQUIRED += rust_target

CARGO_CHANNEL ?= stable

# Currently unknown, something related to the LED_PORT definition that doesn't
# pass C2Rust's transpilation
BOARD_BLACKLIST := ek-lm4f120xl

include $(RIOTBASE)/Makefile.include
