#include <stdint.h>
#include <linux/bpf.h>
#include <bpf/bpf_helpers.h>
#include "helpers.h"

#define SHARED_KEY 0x50
#define COAP_OPT_FINISH_PAYLOAD  (0x0001)

typedef struct __attribute__((packed)) {
    uint8_t ver_t_tkl;
    uint8_t code;
    uint16_t id;
} coap_hdr_t;

typedef struct {
    uint32_t hdr_p;       /* ptr to raw packet */
    uint32_t token_p;     /* ptr to token      */
    uint32_t payload_p;   /* ptr to payload    */
    uint16_t payload_len; /* length of payload */
    uint16_t options_len; /* length of options */
} bpf_coap_pkt_t;

SEC(".main")
int coap_resp(void *ctx){

    bpf_coap_ctx_t gcoap;
    bpf_coap_pkt_t pkt;
    coap_hdr_t hdr;

    // simulate that the packet contains something
    uint8_t payload[20];
    payload[0] = 0x01;
    payload[1] = 0x02;
    payload[1] = 0x03;

    pkt.hdr_p = (uint32_t) &hdr;
    pkt.payload_p = (uint32_t) &payload;
    pkt.payload_len = 20;
    gcoap.pkt = &pkt;

    // initialize the buffer
    uint8_t buf[80];
    gcoap.buf = (uint8_t *) &buf;
    gcoap.buf_len = 80;


    // Simulate having read some value
    int measurement = 123;
    char stringified[20];

    // Write the measurement
    stringified[0] = '1';
    stringified[0] = '2';
    stringified[0] = '3';
    //size_t str_len = bpf_fmt_u32_dec(stringified,
                                     //measurement);

    /* Format the packet with a 205 code */
    bpf_gcoap_resp_init(&gcoap, (2 << 5) | 5);

    /* Add Text type response header */
    bpf_coap_add_format(&gcoap, 0);
    ssize_t pdu_len = bpf_coap_opt_finish(&gcoap,
                 COAP_OPT_FINISH_PAYLOAD);

    //bpf_print_debug(pdu_len);
    uint8_t *pkt_payload =
        (uint8_t*)(intptr_t)(pkt.payload_p);

    // Write the strintified response into the payload
    if (pkt.payload_len >= 3) {
        bpf_memcpy(pkt_payload, stringified,
                   3);
        return pdu_len + 3;
    }

    return  -1;
}
