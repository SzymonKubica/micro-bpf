#include <stdint.h>
#include "saul.h"
#include "saul_reg.h"


#define SAUL_SENSE_TEMP 130

/**
 * Opaque dummy type saul registration
 */
typedef void bpf_saul_reg_t;


uint32_t temperature_read(void)
{
    // First we read the temperature
    bpf_saul_reg_t *dht_temp;
    dht_temp = saul_reg_find_type(SAUL_SENSE_TEMP);

    phydat_t data;
    uint32_t temperature_data;
    saul_reg_read(dht_temp, &data);
    temperature_data = data.val[0];
    return temperature_data;
}

