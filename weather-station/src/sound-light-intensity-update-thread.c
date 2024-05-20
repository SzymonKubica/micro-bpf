#include <stdint.h>
#include "helpers.h"
#include "constants.h"

/* This program is responsible for periodically reading the values reported by
 * all peripheral sensors conntected the device and updating the latest readings
 * in the global storage.
 */

#define SAUL_SENSE_LIGHT 132
#define SAUL_SENSE_SOUND 133

#define US_PER_MSEC (1000)
#define DELAY (50 * US_PER_MSEC)
#define LOGGING_PERIOD 20 // Every 20 iterations = 20 * 50 [ms] = 1s

inline void wait(uint32_t delay);
uint32_t sensor_processing_update_thread(void *ctx)
{
        bpf_saul_reg_t *photoresistor;
        bpf_saul_reg_t *sound_sensor;

        phydat_t light_intensity_data;
        phydat_t sound_intensity_data;

        uint32_t counter = 0;

        photoresistor = bpf_saul_reg_find_nth(6);
        sound_sensor = bpf_saul_reg_find_nth(5);

        while (1) {
                counter = (counter + 1) % LOGGING_PERIOD;
                bpf_saul_reg_read(photoresistor, &light_intensity_data);
                bpf_saul_reg_read(sound_sensor, &sound_intensity_data);

                uint16_t light_intensity = light_intensity_data.val[0];
                uint16_t sound_intenstiy = sound_intensity_data.val[0];

                bpf_store_global(LIGHT_INTENSITY_STORAGE_INDEX,
                                 light_intensity);
                bpf_store_global(SOUND_INTENSITY_STORAGE_INDEX,
                                 sound_intenstiy);

                if (counter % LOGGING_PERIOD == 0) {
                        bpf_printf("[photoresistor] light intensity: %d\%\n",
                                   light_intensity);
                        bpf_printf("[sound_sensor]  sound intensity: %d dB\n",
                                   sound_intenstiy);
                }

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
