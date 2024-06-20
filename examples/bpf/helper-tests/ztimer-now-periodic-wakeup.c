#include "../helpers.h"
#include <stdint.h>

#define PERIOD_US (250 * 1000)

// For some reason we need to inline these functions, otherwise we
// get memory errors when passing the bpf_saul_reg_t pointer around

void __attribute__((noinline)) led_set_state(bpf_saul_reg_t *led, uint32_t state);
void __attribute__((noinline)) led_turn_off(bpf_saul_reg_t *led);
void __attribute__((noinline)) led_turn_on(bpf_saul_reg_t *led);

// For the vm to pick it up correctly, the main function needs to be at the start of the text section
int test_ztimer_periodic_wakeup(void *ctx)
{
    (void)ctx;

    // Play around with the diodes here:
    bpf_saul_reg_t *led;

    uint32_t last_wakeup = bpf_ztimer_now();

    // Toggle all onboard LEDs in order
    int led_index = 0;
    int iterations = 0;
    int max_iterations = 40;
    while (iterations++ < max_iterations) {
        bpf_ztimer_periodic_wakeup(&last_wakeup, PERIOD_US);
        led = bpf_saul_reg_find_nth(led_index);
        // First turn off the current diode
        led_turn_off(led);
        bpf_printf("Turning LED #%d off\n", led_index);

        // Now increment the diode index and turn it on
        led_index = (led_index + 1) % 3;
        led = bpf_saul_reg_find_nth(led_index);
        led_turn_on(led);
        bpf_printf("Turning LED #%d on\n", led_index);
    }

    return 0;
}

void __attribute__((noinline)) led_set_state(bpf_saul_reg_t *led, uint32_t state)
{
    phydat_t led_state;
    led_state.val[0] = state;
    bpf_saul_reg_write(led, &led_state);
}

void __attribute__((noinline)) led_turn_off(bpf_saul_reg_t *led) { led_set_state(led, 0); }

void __attribute__((noinline)) led_turn_on(bpf_saul_reg_t *led) { led_set_state(led, 1); }
