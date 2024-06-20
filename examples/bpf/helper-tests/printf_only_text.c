#include <stdint.h>
#include <string.h>
#include "../helpers.h"

// This string should go into the .rodata section

int test_printf(void *ctx)
{

    char FMT[] = "printf accepts up to 4 args: %d %d %d %d\n";
    bpf_printf(FMT, 9, 10, 11, 12);

    return 0;
}
