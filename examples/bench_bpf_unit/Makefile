RIOTBASE = ../../RIOT
include $(RIOTBASE)/tests/Makefile.tests_common

USEMODULE += embunit
USEMODULE += bpf
USEMODULE += test_utils_interactive_sync

USEMODULE += ztimer_usec
USEMODULE += saul
USEMODULE += saul_reg
USEMODULE += saul_default
USEMODULE += printf_float

# Set to '1' to use femto-containers or to '0' to compile with old rBPF code.
FEMTO ?= 1

# Set to '1' to use the COQ verified implementation.
BPF_COQ ?= 0

# Set to '1' to get CSV compatible print output
CSV ?= 1

ifeq (1,$(FEMTO))
USEPKG += femto-container
else
USEMODULE += bpf
endif

ifeq (1,$(BPF_COQ))
EXTERNAL_MODULE_DIRS += external
USEMODULE += gen_bpf
else
USEMODULE += bpf
endif

# Number of iterations generated for each instruction
NUM_INSTRUCTIONS ?= 2000
BPF_BRANCHES_ALLOWED = $(NUM_INSTRUCTIONS)

CFLAGS += -I$(CURDIR) -DNUM_INSTRUCTIONS=$(NUM_INSTRUCTIONS) -DBPF_COQ=$(BPF_COQ) -DFEMTO=$(FEMTO)
CFLAGS += -DCSV_OUT=$(CSV)
CFLAGS += -DCONFIG_BPF_BRANCHES_ALLOWED=$(BPF_BRANCHES_ALLOWED) -DFEMTO_CONTAINER_BRANCHES_ALLOWED=$(BPF_BRANCHES_ALLOWED) 

include $(RIOTBASE)/Makefile.include
