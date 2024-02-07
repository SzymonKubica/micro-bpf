#include "bpf/bpfapi/helpers.h"
#include <stdint.h>
#include <string.h>

int temp_read(void *ctx) {
  (void)ctx;
  unsigned type = 0x42; /* Temperature sensor */
  bpf_saul_reg_t *sensor;
  phydat_t measurement;

  /* Find temp sensor */
  sensor = bpf_saul_reg_find_type(type);

  if (!sensor || (bpf_saul_reg_read(sensor, &measurement) < 0)) {
    return 2790; /* random temperature to simulate */
  }
  /** format */
  return measurement.val[0] * 100;
}
