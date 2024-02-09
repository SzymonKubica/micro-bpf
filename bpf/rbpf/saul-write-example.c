#include <stdint.h>
#include <linux/bpf.h>
#include "helpers.h"
#include <bpf/bpf_helpers.h>


SEC(".main")
int saul_diode_0_write(void *ctx)
{
    (void)ctx;

    // Play around with the diodes here:
    bpf_saul_reg_t *diode;
    phydat_t diode_payload;

    // Toggle all onboard LEDs in order
    int diode_index = 0;
    int count = 0;
    diode = bpf_saul_reg_find_nth(diode_index);
    diode_payload.val[0] = 1;
    bpf_saul_reg_write(diode, &diode_payload);
    return (int) diode;
}
