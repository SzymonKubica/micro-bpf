#include <stdint.h>
#include "../helpers.h"

/* This file tests whether the saul_reg helper functions work correctly.
 * It assumes that the VM is running on a RIOT instance which has these modules
 * loaded: saul, saul_reg, saul_default. It also assumes that it is running on
 * an stm32 (board name: nucleo-f439zi) and thus the default SAUL configuration
 * has access to the three on-board leds and the user button switch.
 */

#define SAUL_SENSE_TEMP 130
#define SAUL_SENSE_HUM 131
#define US_PER_SEC (1000 * 1000)
#define DELAY (2 * US_PER_SEC)

#define TEMPERATURE_STORAGE_INDEX 0
#define HUMIDITY_STORAGE_INDEX 1

int test_saul_reg_find(void *ctx)
{
    (void)ctx;

    uint16_t temperature_data[] = {223, 224, 225, 226};
    uint16_t humidity_data[] = {653, 780, 810, 842};
    size_t index = 0;

    while (1) {
        uint32_t start = bpf_ztimer_now();
        bpf_ztimer_periodic_wakeup(&start, DELAY);
        index = (index + 1) % 4;

        uint16_t temp = temperature_data[index];
        uint16_t hum = humidity_data[index];

        bpf_printf("[DHT] Reading values \n");
        bpf_printf("temp: %d.%d°C\n", temp / 10, temp % 10);
        bpf_printf("relative humidity: %d.%d%%\n", hum / 10, hum % 10);

        bpf_store_global(TEMPERATURE_STORAGE_INDEX, temp);
        bpf_store_global(HUMIDITY_STORAGE_INDEX, hum);
    }

    // Unreachable
    return 0;
}
