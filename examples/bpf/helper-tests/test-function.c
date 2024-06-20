#include <stdint.h>
#include <string.h>
#include "../helpers.h"

static int __attribute__((noinline)) helper_function();
int test_printf(void *ctx)
{

    helper_function();
    return 0;
}

static int __attribute__((noinline)) helper_function()
{
    // Here we use the macro to avoid defining the format string explicitly
    bpf_printf("printf accepts up to 4 args: %d %d %d %d\n", 1, 2, 3, 4);

    // We can also use the helper directly, however in that case we need to
    // first declare the char[]
    char fmt[] = "printf accepts up to 4 args: %d %d %d %d\n";
    bpf_printf(fmt, 5, 6, 7, 8);

    // After the latest fixes to the rodata section, direct use of the format
    // string is also possible
    bpf_printf("Here is a number: %d\n", 10);
    return 1;
}

