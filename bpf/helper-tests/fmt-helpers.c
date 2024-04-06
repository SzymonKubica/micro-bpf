#include <stdint.h>
#include "../helpers.h"

int test_fmt(void *ctx)
{

    uint32_t val = 123;
    bpf_printf("Writing %d into buffer\n", val);

    // Initialize the buffer where the integer will be written.
    char buffer[4] = {'_', '_', '_', '_'};

    bpf_printf("Buffer before formatting: [%c, %c, %c, %c]\n", buffer[0],
               buffer[1], buffer[2], buffer[3]);

    // Write the integer to the buffer.
    int chars_written = bpf_fmt_u32_dec((char *)buffer, val);

    bpf_printf("Buffer after formatting: [%c, %c, %c, %c]\n", buffer[0],
               buffer[1], buffer[2], buffer[3]);

    int16_t val2 = -12;

    // We also test the second helper here, for integers that need not be
    // unsigned.
    char buffer2[4] = {'_', '_', '_', '_'};

    bpf_printf("Buffer before formatting: [%c, %c, %c, %c]\n", buffer2[0],
               buffer2[1], buffer2[2], buffer2[3]);

    // Write the integer to the buffer.
    int chars_written2 = bpf_fmt_s16_dfp((char *)buffer2, val2, 2);

    bpf_printf("Buffer after formatting: [%c, %c, %c, %c]\n", buffer2[0],
               buffer2[1], buffer2[2], buffer2[3]);

    return chars_written + chars_written2;
}
