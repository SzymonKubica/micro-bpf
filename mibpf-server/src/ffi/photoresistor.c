#include "log.h"
#include "periph/adc.h"
#include "phydat.h"
#include "saul.h"
#include "saul_reg.h"

/* This module implements a SAUL registry entry for the photoresistor allowing
 * for measuring the light intensity values as percentages of the detectable range.
 */

const uint32_t MINIMUM_ADC_VALUE = 12;
const uint32_t MAXIMUM_ADC_VALUE = 1023;

typedef struct photoresitor {
    uint32_t adc_index;
} photoresistor_t;

#define RES ADC_RES_10BIT
#define PHOTORESISTOR_ADC_INDEX 5

uint32_t read_light_intensity(unsigned adc_index) {
    unsigned char adc = ADC_LINE(adc_index);
    int adc_value = adc_sample(adc, RES);
    LOG_DEBUG("raw ADC value: %d\n", adc_value);

    return (adc_value - MINIMUM_ADC_VALUE) * 100 / (MAXIMUM_ADC_VALUE - MINIMUM_ADC_VALUE);
}

int saul_photoresistor_read(const void * dev, phydat_t *res)
{
    photoresistor_t *sensor = (photoresistor_t *)dev;
    res->val[0] = (int16_t) (read_light_intensity(sensor->adc_index));
    res->unit = UNIT_PERCENT;
    res->scale = 0;
    return 1;
}


// A static instance of the light intensity sensor that is used by SAUL registry.
static photoresistor_t saul_dev = {
    .adc_index = PHOTORESISTOR_ADC_INDEX,
};

static saul_driver_t photoresistor_saul_driver = {
    .read =  saul_photoresistor_read,
    .write = saul_write_notsup,
    .type = SAUL_SENSE_LIGHT,
};

static saul_reg_t photoresistor_saul_reg = {
    .name = "photoresistor",
    .dev = &saul_dev,
    .driver = &photoresistor_saul_driver,
};

void photoresistor_saul_register(void) { saul_reg_add(&photoresistor_saul_reg); }
