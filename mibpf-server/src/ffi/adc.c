#include "periph/adc.h"
#include "ztimer.h"
#include "log.h"

/* This module allows for initialising ADC analog input pins and
 * reading from them.
 */

#define RES ADC_RES_10BIT

uint32_t initialise_adc(unsigned adc_index)
{
    if (adc_init(ADC_LINE(adc_index)) < 0) {
        LOG_DEBUG("[adc] Initialization of ADC_LINE(%u) failed\n", adc_index);
        return 1;
    } else {
        LOG_DEBUG("[adc] Successfully initialized ADC_LINE(%u)\n", adc_index);
    }
    return 0;
}

uint32_t read_adc(unsigned adc_index)
{
    unsigned char adc = ADC_LINE(adc_index);
    return adc_sample(adc, RES);

}

#define ADC_NUMOF 7
#define DELAY_MS        1000U

int test_adc(void)
{
    int sample = 0;

    puts("\nRIOT ADC peripheral driver test\n");
    puts("This test will sample all available ADC lines once every 100ms with\n"
         "a 10-bit resolution and print the sampled results to STDIO\n\n");

    /* initialize all available ADC lines */
    for (unsigned i = 0; i < ADC_NUMOF; i++) {
        if (adc_init(ADC_LINE(i)) < 0) {
            printf("Initialization of ADC_LINE(%u) failed\n", i);
            return 1;
        } else {
            printf("Successfully initialized ADC_LINE(%u)\n", i);
        }
    }

    while (1) {
        for (unsigned i = 0; i < ADC_NUMOF; i++) {
            sample = adc_sample(ADC_LINE(i), RES);
            if (sample < 0) {
                printf("ADC_LINE(%u): selected resolution not applicable\n", i);
            } else {
                printf("ADC_LINE(%u): %i\n", i, sample);
            }
        }
        ztimer_sleep(ZTIMER_MSEC, DELAY_MS);
    }

    return 0;
}

