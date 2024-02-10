#include <stdint.h>
#include <string.h>
#include "bpf/bpfapi/helpers.h"


int test_printf(void *ctx)
{

    // Here we use the macro to avoid defining the format string explicitly
    print("printf accepts up to 4 args: %d %d %d %d\n", 1, 2, 3, 4);

    // We can also use the helper directly, however in that case we need to
    // first declare the char[]
    char fmt[] = "printf accepts up to 4 args: %d %d %d %d\n";
    bpf_printf(fmt, 5, 6, 7, 8);
    return 0;
}
