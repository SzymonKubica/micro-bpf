#include <stdint.h>
#include <linux/bpf.h>

#include <bpf/bpf_helpers.h>
#include "helpers.h"

/* This example tests whether the bpf_saul_reg_read helper works correctly.
 * It till print a message to the shell: "[DEBUG] <user-button-status>"
 * One can test it by first executing the program when the button isn't pressed,
 * the message should be: "[DEBUG] 0", if the button is held down, the message
 * will be: "[DEBUG] 1". This will indicate that the value of the button has been
 * correctly read.
 * This assumes that the board has an on-board user button (e.g. stm32 nucleo)
 * and that it has been registered into SAUL under index 3.
 */

SEC(".main")
int saul_reg_read(void *ctx)
{
    (void)ctx;

    int user_button_index = 3;
    bpf_saul_reg_t *user_button;
    phydat_t button_status;


    user_button = bpf_saul_reg_find_nth(user_button_index);
    bpf_saul_reg_read(user_button, &button_status);
    bpf_print_debug(button_status.val[0]);
    return 0;
}
