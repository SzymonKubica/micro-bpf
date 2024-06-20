#include <stdio.h>
// This string should go into the .rodata section
const char FMT[] = "i: %d, %d\n";

int test_printf(void)
{

    for (int i = 0; i < 30; i++) {
        printf(FMT, i, i);
    }

    return 0;
}

