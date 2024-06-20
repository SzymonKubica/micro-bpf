#include <stdint.h>
#include "../helpers.h"

/* This file tests whether the saul_reg helper functions work correctly.
 * It assumes that the VM is running on a RIOT instance which has these modules
 * loaded: saul, saul_reg, saul_default. It also assumes that it is running on
 * an stm32 (board name: nucleo-f439zi) and thus the default SAUL configuration
 * has access to the three on-board leds and the user button switch.
 */


int test_saul_reg_find(void *ctx)
{
    (void)ctx;

    bpf_saul_reg_t *diode0;
    bpf_saul_reg_t *diode1;
    bpf_saul_reg_t *diode2;
    phydat_t diode_payload;

    // First get pointers to the device drivers for the on-board LEDS.
    diode0 = bpf_saul_reg_find_nth(0);
    diode1 = bpf_saul_reg_find_nth(1);
    diode2 = bpf_saul_reg_find_nth(2);

    // Payload tells diodes to turn on.
    diode_payload.val[0] = 1;

    bpf_saul_reg_write(diode0, &diode_payload);
    bpf_saul_reg_write(diode1, &diode_payload);
    bpf_saul_reg_write(diode2, &diode_payload);

    // Now we want to test finding SAUL devices by their type.
    bpf_saul_reg_t *user_button;
    user_button = bpf_saul_reg_find_type(129);
    phydat_t button_state;

    // Read the state of the button.
    // Normally it should write 0 into val[0] of the button state.
    // If this script is executed while the button is being held down,
    // it should write 1 into val[0].
    bpf_saul_reg_read(user_button, &button_state);

    bpf_printf("Button state: %d\n", button_state.val[0]);

    return 0;
}
