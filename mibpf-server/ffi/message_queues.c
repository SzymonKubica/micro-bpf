#include "suit/transport/coap.h"
#include <stdint.h>
#include <stdio.h>

#define MAIN_QUEUE_SIZE (8)

static msg_t _main_msg_queue[MAIN_QUEUE_SIZE];

void do_gnrc_msg_queue_init(void) {
  /* the shell contains commands that receive packets via GNRC and thus
     needs a msg queue (for e.g. ping command) */
  msg_init_queue(_main_msg_queue, MAIN_QUEUE_SIZE);
  puts("GNRC msg queue initialized");
}
