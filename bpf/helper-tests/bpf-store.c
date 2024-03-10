
#include "../helpers.h"

int test_bpf_store() {
    bpf_store_global(1, 2);

    int value = 0;
    bpf_fetch_global(1, &value);

    bpf_printf("Value: %d\n", value);
    return 0;

}
