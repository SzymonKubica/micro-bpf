#include <stdint.h>
#include <linux/bpf.h>

#include <bpf/bpf_helpers.h>
#include "helpers.h"

SEC(".main")
int saul_diode_0_write(void *ctx)
{
    (void)ctx;

    // Play around with the diodes here:
    bpf_saul_reg_t *diode;
    phydat_t diode_payload;

    // Toggle all onboard LEDs in order
    int count = 0;
    int SAUL_ACT_SWITCH = 68;
    diode = bpf_saul_reg_find_type(SAUL_ACT_SWITCH);
    diode_payload.val[0] = 1;
    bpf_saul_reg_write(diode, &diode_payload);
    return 0;
}
