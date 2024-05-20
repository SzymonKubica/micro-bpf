#include <stdint.h>
#include "helpers.h"
#include "constants.h"

/* This program is responsible for periodically reading the values reported by
 * all peripheral sensors conntected the device and updating the latest readings
 * in the global storage.
 */

#define SAUL_SENSE_TEMP 130
#define SAUL_SENSE_HUM 131
#define SAUL_SENSE_LIGHT 132
#define SAUL_SENSE_SOUND 133

#define US_PER_SEC (1000 * 1000)
#define DELAY (2 * US_PER_SEC)



uint32_t sensor_processing_update_thread(void *ctx)
{
    bpf_saul_reg_t *dht_temp;
    bpf_saul_reg_t *dht_hum;
    bpf_saul_reg_t *user_button;
    phydat_t temperature_data;
    phydat_t humidity_data;

    while (1) {
        dht_temp = bpf_saul_reg_find_type(SAUL_SENSE_TEMP);
        dht_hum = bpf_saul_reg_find_type(SAUL_SENSE_HUM);
        user_button = bpf_saul_reg_find_nth(3);

        bpf_saul_reg_read(dht_temp, &temperature_data);

        // We neet do wait at least 2 seconds between subsequent dht readings.
        uint32_t start = bpf_ztimer_now();
        bpf_ztimer_periodic_wakeup(&start, DELAY);

        bpf_saul_reg_read(dht_hum, &humidity_data);

        uint16_t temp = temperature_data.val[0];
        uint16_t hum = humidity_data.val[0];

        bpf_printf("[DHT] Reading values \n");
        bpf_printf("temp: %d.%d°C\n", temp / 10, temp % 10);
        bpf_printf("relative humidity: %d.%d%%\n", hum / 10, hum % 10);

        bpf_store_global(TEMPERATURE_STORAGE_INDEX, temp);
        bpf_store_global(HUMIDITY_STORAGE_INDEX, hum);

        // We neet do wait before going into the next iteration as well
        start = bpf_ztimer_now();
        bpf_ztimer_periodic_wakeup(&start, DELAY);
    }

    // Unreachable
    return 0;

}
