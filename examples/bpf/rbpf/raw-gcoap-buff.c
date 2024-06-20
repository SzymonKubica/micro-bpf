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
int coap_raw_buf(void *ctx)
{
    uint8_t *buf = (uint8_t *)ctx;
    print("address: %d\n", buf);
    print("pointer size: %d\n", sizeof(uint8_t *));
    print("pointer size: %d\n", sizeof(uint64_t));
    print("size_t size: %d\n", sizeof(size_t));
    print("data: %d\n", buf[0]);
    return 0;
}
