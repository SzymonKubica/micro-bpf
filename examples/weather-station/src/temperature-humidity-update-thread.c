#include "constants.h"
#include "helpers.h"
#include <stdint.h>

/* This program is responsible for periodically reading the values reported by
 * all peripheral sensors conntected the device and updating the latest readings
 * in the global storage.
 */

#define SAUL_SENSE_TEMP 130
#define SAUL_SENSE_HUM 131

#define US_PER_SEC (1000 * 1000)
// When interfacing with a DHT sensor, the subsequent measurements need to be at
// least 2 seconds apart as enforced by the communication standard with the
// sensor.
#define DELAY (2 * US_PER_SEC)

inline void wait(uint32_t delay);
void log_readings(uint32_t dht_index, phydat_t temperature, phydat_t humidity);
void store_measurements(uint32_t dht_index, phydat_t temperature,
                        phydat_t humidity);

uint32_t sensor_processing_update_thread(void *ctx)
{
        // We have two DHT22 connected to the device -> indoor and outdoor
        // sensors.
        bpf_saul_reg_t *dht1_temp;
        bpf_saul_reg_t *dht1_hum;
        bpf_saul_reg_t *dht2_temp;
        bpf_saul_reg_t *dht2_hum;

        // Get access to the sensors using their global SAUL IDs.
        dht1_temp = bpf_saul_reg_find_nth(DHT1_TEMP_SAUL_INDEX);
        dht1_hum = bpf_saul_reg_find_nth(DHT1_HUM_SAUL_INDEX);
        dht2_temp = bpf_saul_reg_find_nth(DHT2_TEMP_SAUL_INDEX);
        dht2_hum = bpf_saul_reg_find_nth(DHT2_HUM_SAUL_INDEX);

        while (1) {
                phydat_t temp_data[2];
                phydat_t hum_data[2];

                bpf_saul_reg_read(dht1_temp, &temp_data[0]);
                wait(DELAY);
                bpf_saul_reg_read(dht2_temp, &temp_data[1]);

                // We neet do wait at least 2 seconds between subsequent dht
                // readings. Given that we have the two sensors, we can measure
                // both temperatures at the same time and then wait 2seconds
                // before measuring the two humidities
                wait(DELAY);

                bpf_saul_reg_read(dht1_hum, &hum_data[0]);
                wait(DELAY);
                bpf_saul_reg_read(dht2_hum, &hum_data[1]);

                bpf_printf("[dht] Collected sensor readings. \n");
                log_readings(1, temp_data[0], hum_data[0]);
                log_readings(2, temp_data[1], hum_data[1]);

                store_measurements(1, temp_data[0], hum_data[0]);
                store_measurements(2, temp_data[1], hum_data[1]);

                // We need to wait before the next iteration as well.
                wait(DELAY);
        }

        // Unreachable
        return 0;
}

inline void wait(uint32_t delay)
{
        uint32_t start = bpf_ztimer_now();
        bpf_ztimer_periodic_wakeup(&start, DELAY);
}

void log_readings(uint32_t dht_index, phydat_t temperature, phydat_t humidity)
{
        uint16_t temp = temperature.val[0];
        uint16_t hum = humidity.val[0];
        bpf_printf("[dht%d] temperature: %d.%d°C\n", dht_index, temp / 10,
                   temp % 10);
        bpf_printf("[dht%d] humidity:    %d.%d%%\n", dht_index, hum / 10,
                   hum % 10);
}

void store_measurements(uint32_t dht_index, phydat_t temperature,
                        phydat_t humidity)
{
        uint32_t temperature_storage_indices[] = {DHT1_TEMP_STORAGE_INDEX,
                                                  DHT2_TEMP_STORAGE_INDEX};
        uint32_t humidity_storage_indices[] = {DHT1_HUM_STORAGE_INDEX,
                                               DHT2_HUM_STORAGE_INDEX};

        uint16_t temp = temperature.val[0];
        uint16_t hum = humidity.val[0];
        bpf_store_global(temperature_storage_indices[dht_index - 1], temp);
        bpf_store_global(humidity_storage_indices[dht_index - 1], hum);
}
