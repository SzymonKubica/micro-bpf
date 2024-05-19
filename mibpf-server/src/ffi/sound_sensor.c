#include "log.h"
#include "periph/adc.h"
#include "ztimer.h"
#include <stdio.h>

/* This module implements functions to interact with the sound sensor (KY037)
 * connected to one of the analog input pins.
 *
 * It allows for initialising the given pin as ADC and reading the sound
 * intensity value in decibels. Note that it uses a rather crude approach for
 * calculating the sound intensity, as it measures the peak-to-peak difference
 * over a given period and from that uses rescaling to get the value into the
 * range between 49.5 and 90 [db]
 */

#define RES ADC_RES_10BIT
#define DELAY_MS 50U

float map_range(float x, float in_min, float in_max, float out_min,
                float out_max);

int initialise_adc(unsigned adc_index)
{
    if (adc_init(ADC_LINE(adc_index)) < 0) {
        LOG_DEBUG("[sound sensor] Initialization of ADC_LINE(%u) failed\n",
                  adc_index);
        return 1;
    } else {
        LOG_DEBUG("[sound sensor] Successfully initialized ADC_LINE(%u)\n",
                  adc_index);
    }
    return 0;
}

float map_range(float x, float in_min, float in_max, float out_min,
                float out_max);

uint32_t read_db(unsigned adc_index)
{
    uint32_t sample = 0;

    unsigned char adc = ADC_LINE(adc_index);

    sample = adc_sample(adc, RES);

    uint32_t start = ztimer_now(ZTIMER_MSEC);
    uint32_t signal_max = 0;
    uint32_t signal_min = 1023;
    float peak_to_peak = 0;

    uint32_t sample_window = 50;
    while (ztimer_now(ZTIMER_MSEC) - start < sample_window) {
        sample = adc_sample(ADC_LINE(0), RES);
        if (sample < signal_min) {
            signal_min = sample;
        }
        if (sample > signal_max) {
            signal_max = sample;
        }
    }
    peak_to_peak = signal_max - signal_min;
    uint32_t db = (uint32_t)map_range(peak_to_peak, 20, 900, 49.5, 90);
    LOG_DEBUG("[sound sensor] Sound intensity: %d \n", db);
    return db;
}

float map_range(float x, float in_min, float in_max, float out_min,
                float out_max)
{
    return (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min;
}