#include "bpf/bpfapi/helpers.h"
#include <stdint.h>

#define PERIOD_US (1000 * 1000)

const char message[] =
    "AD3Awn4kb6FtcsyE0RU25U7f55Yncn3LP3oEx9Gl4qr7iDW7I8L6Pbw9jNnh0sE4DmCKuc"
    "d1J8I34vn31W924y5GMS74vUrZQc08805aj4Tf66HgL1cO94os10V2s2GDQ825yNh9Yuq3"
    "QHcA60xl31rdA7WskVtCXI7ruH1A4qaR6Uk454hm401lLmv2cGWt5KTJmr93d3JsGaRRPs"
    "4HqYi4mFGowo8fWv48IcA3N89Z99nf0A0H2R6P0uI4Tir682Of3Rk78DUB2dIGQRRpdqVT"
    "tLhgfET2gUGU65V3edSwADMqRttI9JPVz8JS37g5QZj4Ax56rU1u0m0K8YUs57UYG5645n"
    "byNy4yqxu7";

typedef struct {
  int length;
  __bpf_shared_ptr(void *, payload); /**< Opaque pointer to the payload */
} context_t;


uint32_t fletcher32_bench(void *ctx)
{
    context_t *context = (context_t *)ctx;

    uint8_t *payload = context->payload;
    return (uint32_t) payload[0];
    // We start here to ensure that the entire body of the algorithm is counted.
    uint16_t *data = (uint16_t *)message;

    size_t message_len = 366;
    size_t len = (message_len + 1) & ~1; /* Round up len to words */

    // Made them volatile to avoid optimization
    volatile uint32_t c0 = 0;
    volatile uint32_t c1 = 0;

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
