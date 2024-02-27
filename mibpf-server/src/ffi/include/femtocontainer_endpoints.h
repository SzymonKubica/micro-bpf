#include "femtocontainer/femtocontainer.h"
#include "femtocontainer/shared.h"
#include "fmt.h"
#include "log.h"
#include "suit/storage.h"
#include "suit/storage/ram.h"
#include "suit/transport/coap.h"
#include <stdint.h>
#include <stdlib.h>

typedef struct {
    coap_pkt_t *pdu;
    uint8_t *buf;
    size_t len;
} pkt_buf;

uint32_t execute_femtocontainer_vm(uint8_t *payload, size_t payload_len,

                                   char *location, int64_t *return_value);

uint32_t execute_fc_vm_on_coap_pkt(pkt_buf *ctx, char *location,
                                   uint64_t *return_value);

void copy_packet(pkt_buf *ctx, uint8_t *mem);
