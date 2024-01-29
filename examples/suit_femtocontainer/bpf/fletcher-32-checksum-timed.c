#include "bpf/bpfapi/helpers.h"
#include <stdint.h>
#include <string.h>

#define PERIOD_US (1000 * 1000)

int fletcher32_bench(void *ctx)
{
    (void)ctx;

    // This message was copied from somewhere in riot to replicate their
    // workflow exactly.
    char *message =
        "AD3Awn4kb6FtcsyE0RU25U7f55Yncn3LP3oEx9Gl4qr7iDW7I8L6Pbw9jNnh0sE4DmCKuc"
        "d1J8I34vn31W924y5GMS74vUrZQc08805aj4Tf66HgL1cO94os10V2s2GDQ825yNh9Yuq3"
        "QHcA60xl31rdA7WskVtCXI7ruH1A4qaR6Uk454hm401lLmv2cGWt5KTJmr93d3JsGaRRPs"
        "4HqYi4mFGowo8fWv48IcA3N89Z99nf0A0H2R6P0uI4Tir682Of3Rk78DUB2dIGQRRpdqVT"
        "tLhgfET2gUGU65V3edSwADMqRttI9JPVz8JS37g5QZj4Ax56rU1u0m0K8YUs57UYG5645n"
        "byNy4yqxu7";

    // We start here to ensure that the entire body of the algorithm is counted.
    uint16_t *data = (uint16_t *)message;

    // Algorithm needs the length in words
    uint32_t len = strlen(message) / 2;

    volatile uint32_t c0 = 0;
    volatile uint32_t c1 = 0;

    /* We similarly solve for n > 0 and n * (n+1) / 2 * (2^16-1) < (2^32-1)
     * here.
     */
    /* On modern computers, using a 64-bit c0/c1 could allow a group size of
     * 23726746. */
    uint32_t start = bpf_ztimer_now();
    for (c0 = c1 = 0; len > 0;) {
        uint32_t blocklen = len;
        if (blocklen > 360 * 2) {
            blocklen = 360 * 2;
        }
        len -= blocklen;
        for (uint32_t i = 0; i < blocklen; i += 2) {
            char c = *(data);
            c0 = c0 + c;
            c1 = c1 + c0;
        }
        c0 = c0 % 65535;
        c1 = c1 % 65535;
    }
    uint32_t checksum = (c1 << 16 | c0);
    uint32_t end = bpf_ztimer_now();

    // We return the computation time in microseconds
    return end - start;
}
