
#include "../helpers.h"

int test_bpf_store() {
    // First check the value to investigate that the storage
    // is empty (upon reruning the program it shouldn't be the case
    // as the number should have been written into the storage)
    int value = 0;
    bpf_fetch_global(1, &value);
    bpf_printf("Value: %d\n", value);

    bpf_store_global(1, 2);

    bpf_fetch_global(1, &value);

    bpf_printf("Value: %d\n", value);
    // On subsequent runs, after the value has been set in the storage this
    // program should print 2 twice indicating that the value was there in
    // the storage before the current execution (i.e. it was set by the previous
    // invocation of the VM)
    return 0;

}
