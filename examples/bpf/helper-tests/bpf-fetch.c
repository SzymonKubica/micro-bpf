
#include "../helpers.h"

int test_bpf_store() {
    // First check the value to investigate that the storage
    // is empty (upon reruning the program it shouldn't be the case
    // as the number should have been written into the storage)
    int value = 0;
    bpf_fetch_global(0, &value);
    bpf_printf("Value: %d\n", value);

    return value;

}
