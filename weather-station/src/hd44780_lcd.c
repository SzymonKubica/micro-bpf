#include <stdint.h>
#include "helpers.h"
#include "constants.h"
#define NO_INPUT 4
#define INTERVAL 50
#define DISPLAY_BUTTONS_ADC 2

#define PERIOD_US (1000 * 1000)
char MSG_1[] = "This is a test";
char MSG_2[] = "Weather Station";
char MSG_3[] = "  -- 2.0 --";

int display_update(void *ctx)
{
        uint32_t start = bpf_ztimer_now();
        uint64_t dev = bpf_hd44780_init();
        bpf_hd44780_clear(dev);
        bpf_hd44780_print(dev, MSG_1);

        while (1) {
                bpf_ztimer_periodic_wakeup(&start, INTERVAL);
                uint32_t x = bpf_keypad_get_input(DISPLAY_BUTTONS_ADC);
                if (x != NO_INPUT) {
                        bpf_hd44780_clear(dev);
                        bpf_hd44780_print(dev, "Keypress registered");
                }
        }
}
