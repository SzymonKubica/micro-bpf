#include <stdint.h>
#include <string.h>
#include "../helpers.h"

int test_printf(void *ctx)
{

    // Here we use the macro to avoid defining the format string explicitly
    /*
    char bad_str[] =
        "Here is a really large format string that is supposed to crash the VM."
        "The reason being that the string is allocated on the stack and the"
        "stack is only 512B long. Becuause of this, if we make this string long"
        "enough, it will fill up the whole stack and cause the VM to crash. In"
        " order to test this, we declare a huge string and then try to use it "
        " as a format string %d. Duplicating this message should do the trick"
        "Here is a really large format string that is supposed to crash the VM."
        "The reason being that the string is allocated on the stack and the"
        "stack is only 512B long. Becuause of this, if we make this string long"
        "enough, it will fill up the whole stack and cause the VM to crash. In"
        " order to test this, we declare a huge string and then try to use it "
        " as a format string %d. Duplicating this message should do the trick";
    */
    char bad_str[] = "Made the string shorter %d %d";
    bpf_printf(bad_str, 1234, 5678);
    return 0;
}
