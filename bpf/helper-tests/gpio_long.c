#include <stdint.h>
#include <linux/bpf.h>
#include "../helpers.h"
#include <bpf/bpf_helpers.h>

int gpio_write(void *ctx)
{
    (void)ctx;

    // PA7 corresponds to D12 to which LED is connected
    // Port A
    uint32_t port_a = 0;
    // Pin 7
    uint32_t pin = 6;

    // PF13 corresponds to D7 where the microphone send digital output
    // Port F
    uint32_t port_f = 5;
    // Pin 13
    uint32_t pin_2 = 13;

    // We read the microphone value until sound is detected.
    // After this we turn on the LED and terminate

    // We do this 5 times to make a long running VM.
    // Toggle the led
    uint32_t value = 128;

    while (1) {
        uint64_t mic_value = 0;
        while (!mic_value) {
            mic_value = bpf_gpio_read_input(port_f, pin_2);
        }
        mic_value = 0;

        if (bpf_gpio_read_raw(port_a, pin)) {
            value = 0;
        } else {
            value = 128;
        }
        bpf_gpio_write(port_a, pin, value);
    }
    return (int)value;
}