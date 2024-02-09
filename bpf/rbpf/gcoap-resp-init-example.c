
#include <stdint.h>
#include <linux/bpf.h>

#include <bpf/bpf_helpers.h>
#include "helpers.h"
#include "shared.h"



typedef struct {
    uint32_t hdr_p;       /* ptr to raw packet */
    uint32_t token_p;     /* ptr to token      */
    uint32_t payload_p;   /* ptr to payload    */
    uint16_t payload_len; /* length of payload */
    uint16_t options_len; /* length of options */
} bpf_coap_pkt_t;


SEC(".main")
int gcoap_resp_init_test(void *ctx)
{
    (void)ctx;

    bpf_coap_ctx_t coap_ctx;
    bpf_coap_pkt_t pkt;
    pkt.payload_len = 50;
    coap_ctx.pkt = &pkt;
    coap_ctx.buf_len = 20;
    uint8_t buf[20];
    coap_ctx.buf =(uint8_t *) &buf;

    bpf_gcoap_resp_init(&coap_ctx, 0);
    bpf_coap_pkt_t *pkt_after = (bpf_coap_pkt_t *)coap_ctx.buf;
    bpf_print_debug(pkt_after->payload_len);
    return 0;
}
