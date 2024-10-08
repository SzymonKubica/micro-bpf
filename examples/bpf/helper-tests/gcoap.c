#include <stdint.h>
#include "../helpers.h"

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

int coap_test(bpf_coap_ctx_t *gcoap)
{

    bpf_coap_pkt_t *pkt = gcoap->pkt;
    uint32_t counter = 123;

    char stringified[20];
    size_t str_len = bpf_fmt_u32_dec(stringified, counter);

    // The coap helpers modify the packet, as a consequence the length of the
    // payload changes. We log it to the console to ensure that the helper
    // functions correctly invoke the underlying coap functions.
    bpf_printf("Payload length: %d\n", pkt->payload_len);

    unsigned code = (2 << 5) | 5;
    bpf_printf("Writing response code: %d\n", code);

    bpf_gcoap_resp_init(gcoap, code);

    // Check that the code has been written correctly
    coap_hdr_t *hdr = (coap_hdr_t *)(intptr_t)(pkt->hdr_p);
    bpf_printf("Response code: %d\n", hdr->code);

    bpf_printf("Payload length: %d\n", pkt->payload_len);
    // Adding format adds an option to the packet. We should expect the number
    // of options to increase by 1.
    bpf_printf("Options length before bpf_coap_add_format: %d\n", pkt->options_len);
    bpf_coap_add_format(gcoap, 0);

    bpf_printf("Options length after bpf_coap_add_format: %d\n", pkt->options_len);

    // The coap_opt_finish writes 0xFF at the current payload pointer and then
    // decrements the payload length by 1.
    bpf_printf("Payload length before bpf_coap_opt_finish: %d\n", pkt->payload_len);
    ssize_t pdu_len = bpf_coap_opt_finish(gcoap, COAP_OPT_FINISH_PAYLOAD);
    bpf_printf("Payload length after bpf_coap_opt_finish: %d\n", pkt->payload_len);

    uint8_t *payload = (uint8_t *)(pkt->payload_p);

    if (pkt->payload_len >= str_len) {
        bpf_memcpy(payload, stringified, str_len);
        bpf_printf("Payload length + written string: %d\n", pdu_len + str_len);
        //for (int i = 0; i < str_len; i++) {
        //    bpf_printf("Payload[%d]: %d\n", i, payload[i]);
        //}
        return pdu_len + str_len;
    }

    return 100;
}
