#include <stdint.h>
#include "helpers.h"

#define SAUL_SENSE_TEMP 130
#define TEMP_DATA_START 0
#define TEMP_DATA_PTR 13
#define TEMP_NEW_DATA 15
#define TEMP_STORAGE_SLOTS 12

#define ENABLE_DEBUG 0
#define DEBUG(...)                                                             \
    do {                                                                       \
        if (ENABLE_DEBUG) {                                                    \
            bpf_printf(__VA_ARGS__);                                           \
        }                                                                      \
    } while (0)

/* This program computes a moving average of temperature readings
 * by interacting with the global storage provided for eBPF programs.
 *
 * It reads temperature data from the DHT22 sensor and then computes and updates
 * the moving average of the temperature readings.
 *
 * This is implemented by having TEMP_STORAGE_SLOTS slots for temperature
 * readings and each time this program is run we update a different slot in the
 * storage using round-robin fashion. The 'pointer' telling us which slot is to
 * be updated this time is stored under TEMP_DATA_PTR in the
 * global storage. Each time we run the program this pointer is incremented
 * modulo TEMP_STORAGE_SLOTS and then saved in the storage, while the
 * storage slot pointed to by the temperature pointer is updated with the latest
 * temperature reading.
 *
 * In order to get the moving average, the program reads all slots at the end
 * and computes the average which is then retured from the program. The actual
 * temperature value is multiplied by 10 to allow for 1 decimal place of
 * precision.
 */

uint32_t sensor_processing_from_storage(void *ctx)
{

    // First we read the temperature
    bpf_saul_reg_t *dht_temp;
    uint32_t temp;
    // A separate thread updates the temperature in this slot
    bpf_fetch_global(TEMP_NEW_DATA, &temp);

    // Update the temperature storage
    uint32_t pointer = 0;
    bpf_fetch_global(TEMP_DATA_PTR, &pointer);
    bpf_store_global(TEMP_DATA_PTR, (pointer + 1) % TEMP_STORAGE_SLOTS);
    bpf_store_global(TEMP_DATA_START + pointer, temp);

    for (uint32_t i = 1; i < TEMP_STORAGE_SLOTS; i++) {
        uint32_t old_temp = 0;
        uint32_t offset = (pointer + i) % TEMP_STORAGE_SLOTS;
        bpf_fetch_global(TEMP_DATA_START + offset, &old_temp);
        // We need to fill in empty values
        if (old_temp == 0) {
            bpf_store_global(TEMP_DATA_START + offset, temp);
        }
    }

    // Compute the moving average
    uint32_t all_readings[TEMP_STORAGE_SLOTS];
    for (uint32_t i = 0; i < TEMP_STORAGE_SLOTS; i++) {
        uint32_t reading = 0;
        bpf_fetch_global(TEMP_DATA_START + i, &reading);
        all_readings[i] = reading;
    }

    uint32_t sum = 0;
    for (uint32_t i = 0; i < TEMP_STORAGE_SLOTS; i++) {
        sum += all_readings[i];
    }
    DEBUG("[DHT] Fetched temperature history: \n");
    DEBUG("[%d, %d, %d, %d]\n", all_readings[0], all_readings[1],
          all_readings[2], all_readings[3]);

    uint32_t average = sum / TEMP_STORAGE_SLOTS;
    DEBUG("[DHT] Calculated moving average: \n");
    DEBUG("temp: %d.%d°C\n", average / 10, average % 10);
    return average;
}
