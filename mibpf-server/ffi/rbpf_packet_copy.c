#include "log.h"
#include "suit/transport/coap.h"
#include <stdint.h>

typedef struct {
    coap_pkt_t *pdu;
    uint8_t *buf;
    size_t len;
} pkt_buf;

// Copies all contents of the packet under *ctx into the provided memory region.
// It also recalculates pointers inside of that packet struct so that they point
// to correct offsets in the target memory buffer. This function is needed for
// executing the rBPF VM on raw packet data.
void copy_packet(pkt_buf *ctx, uint8_t *mem)
{
    uint64_t *memory_region = (uint64_t *)mem;
    uint8_t *pkt_ptr = (uint8_t *)memory_region;
    // skip two places for the pointers to the packet and the buffer.
    // skip one place for the length
    pkt_ptr += 3 * sizeof(uint64_t);

    // Write the buffer and save its address.
    uint8_t *buf_ptr = pkt_ptr + sizeof(coap_pkt_t);
    memcpy(buf_ptr, ctx->buf, sizeof(*ctx->buf));
    LOG_DEBUG("Buffer size: %d\n", sizeof(*ctx->buf));
    LOG_DEBUG("Original Buffer pointer: %d\n", ctx->buf);
    LOG_DEBUG("Buffer length: %d\n", ctx->len);

    // Before we write the pkt, we need to adjust its header and payload
    // pointers.
    coap_pkt_t *pkt = (coap_pkt_t *)ctx->pdu;
    // The header located at the beginning of the buffer.
    uint8_t *hdr_ptr = buf_ptr;
    memcpy(hdr_ptr, pkt->hdr, sizeof(coap_hdr_t));
    LOG_DEBUG("Original pkt hdr pointer: %d\n", pkt->hdr);

    // Payload starts immediately after the header
    uint8_t *payload_ptr = hdr_ptr + sizeof(coap_hdr_t);
    memcpy(payload_ptr, pkt->payload, pkt->payload_len);
    LOG_DEBUG("Payload length: %d\n", pkt->payload_len);

    pkt->payload = payload_ptr;
    pkt->hdr = (coap_hdr_t *)hdr_ptr;
    // Now we know pointers to header and payload so we can write the pkt info.
    memcpy(pkt_ptr, ctx->pdu, sizeof(coap_pkt_t));
    LOG_DEBUG("coap_pkt_t size: %d\n", sizeof(coap_pkt_t));

    // Now write pointers to the actual places in the array
    memory_region[0] = (uint64_t)pkt_ptr;
    memory_region[1] = (uint64_t)buf_ptr;
    memory_region[2] = (size_t)ctx->len;

    LOG_DEBUG("Buf ptr: %d\n", buf_ptr);
    LOG_DEBUG("Memory region start: %d\n", memory_region);
    LOG_DEBUG("pkt ptr: %d\n", (int)memory_region[0]);
    LOG_DEBUG("buf ptr: %d\n", (int)memory_region[1]);
    LOG_DEBUG("hdr ptr: %d\n", hdr_ptr);
    LOG_DEBUG("payload ptr: %d\n", payload_ptr);
    LOG_DEBUG("buf len: %d\n", (int)memory_region[2]);
}
