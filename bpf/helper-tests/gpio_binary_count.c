#include <stdint.h>
#include <linux/bpf.h>
#include <math.h>
#include "../helpers.h"
#include <bpf/bpf_helpers.h>

#define D7_PORT 5
#define D7_PIN 13

#define D7_PORT 5
#define D7_PIN 13

#define D8_PORT 5
#define D8_PIN 12

#define D5_PORT 4
#define D5_PIN 11

// The LEDs are connected in a way that they are always connected to 5V on
// one end the other end is connected to the digital GPIO, we turn the leds on
// by turning the GPIO off.
#define ON 0
#define OFF 4096

#define PERIOD_US (250 * 1000)

inline void set_led(uint32_t index, uint32_t value)
{
    switch (index) {
    case 0:
        bpf_gpio_write(D5_PORT, D5_PIN, value);
        break;
    case 1:
        bpf_gpio_write(D7_PORT, D7_PIN, value);
        break;
    case 2:
        bpf_gpio_write(D8_PORT, D8_PIN, value);
        break;
    }
}

int gpio_write(void *ctx)
{
    (void)ctx;

    int count = 0;

    uint32_t last_wakeup = bpf_ztimer_now();
    while (count < 128) {
        bpf_ztimer_periodic_wakeup(&last_wakeup, PERIOD_US);
        for (int i = 0; i < 3; i++) {
            if (count & (1 << i)) {
                set_led(i, ON);
            } else {
                set_led(i, OFF);
            }
        }
        bpf_printf("Count: %d\n", count);
        count++;
    }
    return 0;
}
