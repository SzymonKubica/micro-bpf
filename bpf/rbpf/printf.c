#include <linux/bpf.h>
#include <stdint.h>
#include <string.h>
#include <bpf/bpf_helpers.h>
#include "helpers.h"


SEC(".main")
int test_printf(void *ctx)
{
    print("printf accepts up to 4 args: %d %d %d %d\n", 1, 2, 3, 4);
    return 0;
}

