#include <stdint.h>
#include <stddef.h>
#include <bpf.h>
#include <fmt.h>
#include <net/nanocoap.h>
#include <net/gcoap.h>
#include <bpf/store.h>

#define SHARED_KEY 0x50
#define COAP_OPT_FINISH_PAYLOAD (0x0001)

#define TEMPERATURE_STORAGE_START 0
#define TEMPERATURE_STORAGE_END 10
const unsigned SUCCESS_RESPONSE_CODE = (2 << 5) | 5;

typedef struct {
    coap_pkt_t *pdu;
    uint8_t *buf;
    size_t len;
} pkt_buf;

int gcoap_temperature(pkt_buf *ctx)
{
    coap_pkt_t *pdu = ctx->pdu;
    uint8_t *buf = ctx->buf;
    size_t len = ctx->len;

    uint32_t temperature_data[10];
    uint32_t temperature_reading;
    for (uint32_t i = TEMPERATURE_STORAGE_START; i < TEMPERATURE_STORAGE_END; i++) {
        bpf_store_fetch_global(i, &temperature_reading);
        temperature_data[i - TEMPERATURE_STORAGE_START] = temperature_reading;
    }

    uint32_t sum_temperature = 0;
    for (uint32_t i = TEMPERATURE_STORAGE_START; i < TEMPERATURE_STORAGE_END; i++) {
        sum_temperature += temperature_data[i - TEMPERATURE_STORAGE_START];
    }

    uint32_t avg_temperature =
        sum_temperature / (TEMPERATURE_STORAGE_END - TEMPERATURE_STORAGE_START);

    char fmt_buffer[5];

    // -1 means that there is one decimal point.
    size_t str_len = fmt_s16_dfp(fmt_buffer, avg_temperature, -1);

    gcoap_resp_init(pdu, buf, len, SUCCESS_RESPONSE_CODE);

    // Adding format adds an option to the packet. We should expect the number
    // of options to increase by 1.
    coap_opt_add_format(pdu, 0);
    ssize_t pdu_len = coap_opt_finish(pdu, COAP_OPT_FINISH_PAYLOAD);

    uint8_t *payload = (uint8_t *)(pdu->payload);

    if (pdu->payload_len >= str_len) {
        uint32_t start_len = 16;
        uint32_t end_len = 2;
        char fmt[] = "{\"temperature\": }";
        memcpy(payload, fmt, start_len);
        memcpy(payload + start_len, fmt_buffer, str_len);
        memcpy(payload + start_len + str_len, fmt + start_len, end_len);
        // It is very important that the programs modifying response packet
        // buffer return the correct length of the payload. This is because this
        // return value is then used by the server to determine which subsection
        // of the buffer was written to and needs to be sent back to the client.
        return pdu_len + str_len + start_len + end_len;
    }
    return -1;
}

