#include <stdint.h>
#include <linux/bpf.h>
#include "../helpers.h"
#include <bpf/bpf_helpers.h>

#define PERIOD_US (1000 * 1000)
int lcd_test(void *ctx)
{

    uint64_t dev = bpf_hd44780_init();

    uint32_t last_wakeup = bpf_ztimer_now();

    bpf_hd44780_clear(dev);
    bpf_hd44780_print(dev, "This is a test");

    bpf_ztimer_periodic_wakeup(&last_wakeup, PERIOD_US);

    bpf_hd44780_clear(dev);
    bpf_hd44780_print(dev, "Weather Station");
    bpf_hd44780_set_cursor(dev, 0, 1);
    bpf_hd44780_print(dev, "  -- 2.0 --");
    return 0;
}
