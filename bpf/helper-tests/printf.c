#include <stdint.h>
#include <string.h>
#include "../helpers.h"

// This string should go into the .rodata section
const char FMT[] = "printf accepts up to 4 args: %d %d %d %d\n";

int test_printf(void *ctx)
{

    bpf_printf("printf accepts up to 4 args: %d %d %d %d\n", 1, 2, 3, 4);

    // We can also use the helper directly, however in that case we need to
    // first declare the char[]
    char fmt[] = "printf accepts up to 4 args: %d %d %d %d\n";
    bpf_printf(fmt, 5, 6, 7, 8);

    bpf_printf(FMT, 9, 10, 11, 12);

    // After the latest fixes to the rodata section, direct use of the format
    // string is also possible
    bpf_printf("Here is a number: %d\n", 10);
    bpf_printf("Here is another number: %d\n", 12);
    return 0;
}
