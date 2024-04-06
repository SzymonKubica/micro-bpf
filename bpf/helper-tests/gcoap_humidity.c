#include <stdint.h>
#include "../helpers.h"

#define SHARED_KEY 0x50
#define COAP_OPT_FINISH_PAYLOAD (0x0001)

typedef struct {
    uint32_t hdr_p;       /* ptr to raw packet */
    uint32_t payload_p;   /* ptr to payload    */
    uint32_t token_p;     /* ptr to token      */
    uint16_t payload_len; /* length of payload */
    uint16_t options_len; /* length of options */
} bpf_coap_pkt_t;

typedef struct __attribute__((packed)) {
    uint8_t ver_t_tkl;
    uint8_t code;
    uint16_t id;
} coap_hdr_t;


#define HUMIDITY_STORAGE_INDEX 1

int coap_test(bpf_coap_ctx_t *gcoap)
{
    bpf_coap_pkt_t *pkt = gcoap->pkt;
    int humidity = 0;
    bpf_fetch_global(HUMIDITY_STORAGE_INDEX, &humidity);

    char stringified[20];
    // -1 means that there is one decimal point.
    size_t str_len = bpf_fmt_s16_dfp(stringified, humidity, -1);

    unsigned code = (2 << 5) | 5;
    bpf_printf("Writing response code: %d\n", code);
    bpf_gcoap_resp_init(gcoap, (2 << 5) | 5);

    // Check that the code has been written correctly
    coap_hdr_t *hdr = (coap_hdr_t *)(intptr_t)(pkt->hdr_p);
    bpf_printf("Checking response code: %d\n", hdr->code);

    bpf_printf("Payload length: %d\n", pkt->payload_len);
    // Adding format adds an option to the packet. We should expect the number
    // of options to increase by 1.
    bpf_coap_add_format(gcoap, 0);
    ssize_t pdu_len = bpf_coap_opt_finish(gcoap, COAP_OPT_FINISH_PAYLOAD);

    uint8_t *payload = (uint8_t *)(pkt->payload_p);

    bpf_printf("Copying stringified temperature reading payload\n");
    if (pkt->payload_len >= str_len) {
        char start[] = "{\"humidity\": ";
        int start_len = 13;
        char end[] = "}";
        int end_len = 2;
        bpf_memcpy(payload, start, start_len);
        bpf_memcpy(payload+start_len, stringified, str_len);
        bpf_memcpy(payload+start_len+str_len, end, end_len);
        return pdu_len + str_len + start_len + end_len;
    }

    return -1;
}
