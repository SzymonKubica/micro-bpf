#include "bpf/bpfapi/helpers.h"
#include <stdint.h>
#include <string.h>

#define PERIOD_US (1000 * 1000)

int fletcher32_bench(void *ctx)
{
    (void)ctx;

    // Play around with the diodes here:
    bpf_saul_reg_t *diode;
    phydat_t diode_payload;

    uint32_t last_wakeup = bpf_ztimer_now();

    // Toggle all onboard LEDs in order
    int diode_index = 0;
    int count = 0;
    int max_iterations = 100;
    while (count++ < max_iterations) {
        bpf_ztimer_periodic_wakeup(&last_wakeup, PERIOD_US);
        // First turn off the current diode
        diode = bpf_saul_reg_find_nth(diode_index);
        diode_payload.val[0] = 0;
        bpf_saul_reg_write(diode, &diode_payload);
        // Now increment the diode index and turn it on
        diode_index = (diode_index + 1) % 3;
        diode = bpf_saul_reg_find_nth(diode_index);
        diode_payload.val[0] = 1;
        bpf_saul_reg_write(diode, &diode_payload);
    }

    return 0;
}
