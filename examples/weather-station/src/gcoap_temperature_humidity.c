#include <stdint.h>
#include "constants.h"
#include "helpers.h"

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


#define TEMPERATURE_STORAGE_INDEX 15
const unsigned SUCCESS_RESPONSE_CODE = (2 << 5) | 5;

int gcoap_temperature_humidity(bpf_coap_ctx_t *gcoap)
{
    bpf_coap_pkt_t *pkt = gcoap->pkt;

    uint32_t temperature = 0;
    bpf_fetch_global(DHT1_TEMP_STORAGE_INDEX, &temperature);

    char fmt_buffer[5];

    // -1 means that there is one decimal point.
    size_t str_len = bpf_fmt_s16_dfp(fmt_buffer, temperature, -1);

    uint32_t humidity = 0;
    bpf_fetch_global(DHT1_HUM_STORAGE_INDEX, &humidity);

    char fmt_buffer2[5];

    // -1 means that there is one decimal point.
    size_t humidity_len = bpf_fmt_s16_dfp(fmt_buffer2, humidity, -1);

    bpf_printf("Writing response code: %d\n", SUCCESS_RESPONSE_CODE);
    bpf_gcoap_resp_init(gcoap, SUCCESS_RESPONSE_CODE);

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
        char fmt[] = "{\"temperature\": , \"humidity\": }";
        int start_len = 16;
        int middle_len = 14;
        int end_len = 2;

        bpf_memcpy(payload, fmt, start_len);
        bpf_memcpy(payload + start_len, fmt_buffer, str_len);
        bpf_memcpy(payload + start_len + str_len, fmt + start_len, middle_len);
        bpf_memcpy(payload + start_len + str_len + middle_len, fmt_buffer2, humidity_len);
        bpf_memcpy(payload + start_len + str_len + middle_len + humidity_len, fmt + start_len + middle_len, end_len);

        // It is very important that the programs modifying response packet
        // buffer return the correct length of the payload. This is because this
        // return value is then used by the server to determine which subsection
        // of the buffer was written to and needs to be sent back to the client.
        return pdu_len + str_len + start_len + end_len + middle_len + humidity_len;
    }
    return -1;
}
