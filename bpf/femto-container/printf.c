#include <stdint.h>
#include <string.h>
#include "bpf/bpfapi/helpers.h"

int temp_read(void *ctx) {
  (void)ctx;

  bpf_printf("%s", 123, 456, 789, 103);
  return 0;
}
