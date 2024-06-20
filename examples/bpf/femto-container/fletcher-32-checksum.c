#include "bpf/bpfapi/helpers.h"
#include <stdint.h>

typedef struct {
  __bpf_shared_ptr(void *, payload); /**< Opaque pointer to the payload */
  int payload_length;
} context_t;


uint32_t fletcher32_bench(void *ctx)
{
    context_t *context = (context_t *)ctx;

    uint8_t *payload = context->payload;

    uint16_t *data = (uint16_t *)payload;

    size_t len = (context->payload_length + 1) & ~1; /* Round up len to words */

    uint32_t c0 = 0;
    uint32_t c1 = 0;

    for (c0 = c1 = 0; len > 0;) {
        uint32_t blocklen = len;
        if (blocklen > 360 * 2) {
            blocklen = 360 * 2;
        }
        len -= blocklen;
        do {
            c0 = c0 + *data++;
            c1 = c1 + c0;
        } while ((blocklen -= 2));

        c0 = c0 % 65535;
        c1 = c1 % 65535;
    }
    uint32_t checksum = (c1 << 16 | c0);
    return checksum;
}
