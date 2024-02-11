#include <linux/bpf.h>
#include <stdint.h>
#include <string.h>
#include <bpf/bpf_helpers.h>
#include "../../helpers.h"

#define SHARED_KEY 0x50
#define COAP_OPT_FINISH_PAYLOAD (0x0001)

typedef struct {
    uint32_t hdr_p;       /* ptr to raw packet */
    uint32_t token_p;     /* ptr to token      */
    uint32_t payload_p;   /* ptr to payload    */
    uint16_t payload_len; /* length of payload */
    uint16_t options_len; /* length of options */
} bpf_coap_pkt_t;

typedef struct __attribute__((packed)) {
    uint8_t ver_t_tkl;
    uint8_t code;
    uint16_t id;
} coap_hdr_t;

SEC(".main")
int coap_resp(bpf_coap_ctx_t *gcoap)
{
    bpf_coap_pkt_t *pkt = gcoap->pkt;
    uint32_t counter = 123;

    char stringified[20];
    size_t str_len = bpf_fmt_u32_dec(stringified, counter);

    // The coap helpers modify the packet, as a consequence the length of the
    // payload changes. We log it to the console to ensure that the helper
    // functions correctly invoke the underlying coap functions.
    print("Payload length: %d\n", pkt->payload_len);

    // Find out why the stack overflows here

    unsigned code = (2 << 5) | 5;
    print("Writing response code: %d\n", code);

    bpf_gcoap_resp_init(gcoap, (2 << 5) | 5);

    // Check that the code has been written correctly
    coap_hdr_t *hdr = (coap_hdr_t *)(intptr_t)(pkt->hdr_p);
    print("Response code: %d\n", hdr->code);

    print("Payload length: %d\n", pkt->payload_len);
    // Adding format adds an option to the packet. We should expect the number
    // of options to increase by 1.
    print("Options length before bpf_coap_add_format: %d\n", pkt->options_len);
    bpf_coap_add_format(gcoap, 0);

    print("Options length after bpf_coap_add_format: %d\n", pkt->options_len);

    // The coap_opt_finish writes 0xFF at the current payload pointer and then
    // decrements the payload length by 1.
    print("Payload length before bpf_coap_opt_finish: %d\n", pkt->payload_len);
    ssize_t pdu_len = bpf_coap_opt_finish(gcoap, COAP_OPT_FINISH_PAYLOAD);
    print("Payload length after bpf_coap_opt_finish: %d\n", pkt->payload_len);

    uint8_t *payload = (uint8_t *)(intptr_t)(pkt->payload_p);

    if (pkt->payload_len >= str_len) {
        bpf_memcpy(payload, stringified, str_len);
        return pdu_len + str_len;
    }

    return -1;
}
