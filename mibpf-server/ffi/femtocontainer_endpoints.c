#include "include/femtocontainer_endpoints.h"

static uint8_t _stack[512] = {0};

static f12r_t _bpf = {
    .stack_region = NULL,
    .rodata_region = NULL,
    .data_region = NULL,
    .arg_region = NULL,
    .application = NULL,  /**< Application bytecode */
    .application_len = 0, /**< Application length */
    .stack = _stack,
    .stack_size = sizeof(_stack),
    .flags = FC_CONFIG_NO_RETURN,
    // TODO: set branches rem to something sensible
    .branches_remaining =
        100, /**< Number of allowed branch instructions remaining */
};

typedef struct {
    // Need to use this stupid opaque pointer otherwise the address is
    // translated incorrectly.
    __bpf_shared_ptr(void *, payload); /**< Opaque pointer to the payload */
    int payload_length;
} context_t;

// Context struct for handling CoAP packets
typedef struct {
    __bpf_shared_ptr(void *,
                     pkt); /**< Opaque pointer to the coap_pkt_t struct */
    __bpf_shared_ptr(uint8_t *, buf); /**< Packet buffer */
    size_t buf_len;                   /**< Packet buffer length */
} f12r_coap_ctx_t;

uint32_t execute_femtocontainer_vm(uint8_t *payload, size_t payload_len,
                                   char *location, int64_t *return_value)
{
    LOG_DEBUG(
        "[BPF handler]: getting appropriate SUIT backend depending on the "
        "storage "
        "location id. \n");

    suit_storage_t *storage = suit_storage_find_by_id(location);

    assert(storage);

    LOG_DEBUG("[BPF handler]: setting suit storage active location: %s\n",
              location);

    suit_storage_set_active_location(storage, location);
    const uint8_t *mem_region;
    size_t length;

    LOG_DEBUG("[BPF handler]: getting a pointer to the data stored in the SUIT "
              "location. \n");
    suit_storage_read_ptr(storage, &mem_region, &length);

    LOG_DEBUG("[BPF handler]: Application bytecode:\n");
    for (size_t i = 0; i < length; i++) {
        LOG_DEBUG("%02x", mem_region[i]);
        // Add a new line every 8x8 bits -> each eBPF instruction is 64 bits
        // long.
        if (i % 8 == 7) {
            LOG_DEBUG("\n");
        }
    }
    LOG_DEBUG("\n");

    LOG_DEBUG("[BPF handler]: initialising the eBPF application struct\n");
    _bpf.application = mem_region;
    _bpf.application_len = length;

    f12r_mem_region_t mem_context;

    LOG_DEBUG("[BPF handler]: initialising bpf context with payload.\n");
    context_t *bpf_ctx = malloc(sizeof(context_t));
    bpf_ctx->payload = payload;
    bpf_ctx->payload_length = payload_len;
    LOG_DEBUG("Payload pointer: %p \n", (void *)payload);

    // TODO: find out how to set the memory regions correctly
    LOG_DEBUG("[BPF handler]: payload length: %d\n", payload_len);

    // Regions need to be added after the setup so that they are taken into
    // account
    f12r_setup(&_bpf);
    f12r_add_region(&_bpf, &mem_context, payload, payload_len,
                    FC_MEM_REGION_READ | FC_MEM_REGION_WRITE);

    int64_t result = -1;
    printf("[BPF handler]: executing VM\n");
    ztimer_acquire(ZTIMER_USEC);
    ztimer_now_t start = ztimer_now(ZTIMER_USEC);
    // Figure out the size of the context
    int res = f12r_execute_ctx(&_bpf, bpf_ctx, 64, &result);
    ztimer_now_t end = ztimer_now(ZTIMER_USEC);
    uint32_t execution_time = end - start;
    *return_value = result;

    printf("[BPF handler]: Execution complete res=%i, result=%d \nExecution "
           "time: %i [us]\n",
           res, (uint32_t)result, execution_time);

    return execution_time;
}


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
    LOG_INFO("Buffer size: %d\n", sizeof(*ctx->buf));
    LOG_INFO("Original Buffer pointer: %d\n", ctx->buf);
    LOG_INFO("Buffer length: %d\n", ctx->len);

    // Before we write the pkt, we need to adjust its header and payload
    // pointers.
    coap_pkt_t *pkt = (coap_pkt_t *)ctx->pdu;
    // The header located at the beginning of the buffer.
    uint8_t *hdr_ptr = buf_ptr;
    memcpy(hdr_ptr, pkt->hdr, sizeof(coap_hdr_t));
    LOG_INFO("Original pkt hdr pointer: %d\n", pkt->hdr);

    // Payload starts immediately after the header
    uint8_t *payload_ptr = hdr_ptr + sizeof(coap_hdr_t);
    memcpy(payload_ptr, pkt->payload, pkt->payload_len);
    LOG_INFO("Payload length: %d\n", pkt->payload_len);

    pkt->payload = payload_ptr;
    pkt->hdr = (coap_hdr_t *)hdr_ptr;
    // Now we know pointers to header and payload so we can write the pkt info.
    memcpy(pkt_ptr, ctx->pdu, sizeof(coap_pkt_t));
    LOG_INFO("coap_pkt_t size: %d\n", sizeof(coap_pkt_t));

    // Now write pointers to the actual places in the array
    memory_region[0] = (uint64_t)pkt_ptr;
    memory_region[1] = (uint64_t)buf_ptr;
    memory_region[2] = (size_t)ctx->len;

    LOG_INFO("Buf ptr: %d\n", buf_ptr);
    LOG_INFO("Memory region start: %d\n", memory_region);
    LOG_INFO("pkt ptr: %d\n", (int)memory_region[0]);
    LOG_INFO("buf ptr: %d\n", (int)memory_region[1]);
    LOG_INFO("hdr ptr: %d\n", hdr_ptr);
    LOG_INFO("payload ptr: %d\n", payload_ptr);
    LOG_INFO("buf len: %d\n", (int)memory_region[2]);
}

uint32_t execute_fc_vm_on_coap_pkt(uint8_t program, size_t program_len, pkt_buf *ctx,
                                   uint64_t *return_value)
{

    coap_pkt_t *pdu = ctx->pdu;
    uint8_t *buf = ctx->buf;
    size_t len = ctx->len;

    LOG_DEBUG(
        "[BPF handler]: getting appropriate SUIT backend depending on the "
        "storage "
        "location id. \n");

    suit_storage_t *storage = suit_storage_find_by_id(location);

    assert(storage);

    LOG_DEBUG("[BPF handler]: setting suit storage active location: %s\n",
              location);
    suit_storage_set_active_location(storage, location);
    const uint8_t *mem_region;
    size_t length;

    LOG_DEBUG("[BPF handler]: getting a pointer to the data stored in the SUIT "
              "location. \n");
    suit_storage_read_ptr(storage, &mem_region, &length);

    LOG_DEBUG("[BPF handler]: Application bytecode:\n");
    for (size_t i = 0; i < length; i++) {
        LOG_DEBUG("%02x", mem_region[i]);
        // Add a new line every 8x8 bits -> each eBPF instruction is 64 bits
        // long.
        if (i % 8 == 7) {
            LOG_DEBUG("\n");
        }
    }
    LOG_DEBUG("\n");

    LOG_DEBUG("[BPF handler]: initialising the eBPF application struct\n");
    _bpf.application = mem_region;
    _bpf.application_len = length;

    f12r_mem_region_t mem_pdu;
    f12r_mem_region_t mem_pkt;

    f12r_coap_ctx_t bpf_ctx = {
        .pkt = pdu,
        .buf = buf,
        .buf_len = len,
    };
    LOG_DEBUG("[BPF]: executing gcoap handler\n");

    f12r_setup(&_bpf);
    f12r_add_region(&_bpf, &mem_pdu, pdu->hdr, 256,
                    FC_MEM_REGION_READ | FC_MEM_REGION_WRITE);
    f12r_add_region(&_bpf, &mem_pkt, pdu, sizeof(coap_pkt_t),
                    FC_MEM_REGION_READ | FC_MEM_REGION_WRITE);

    int64_t result = -1;
    LOG_DEBUG("[BPF handler]: executing VM\n");
    ztimer_acquire(ZTIMER_USEC);
    ztimer_now_t start = ztimer_now(ZTIMER_USEC);
    // Figure out the size of the context
    int res = f12r_execute_ctx(&_bpf, &bpf_ctx, 64, &result);
    ztimer_now_t end = ztimer_now(ZTIMER_USEC);
    uint32_t execution_time = end - start;
    *return_value = result;

    LOG_DEBUG("[BPF handler]: Execution complete res=%i, result=%d \nExecution "
           "time: %i [us]\n",
           res, (uint32_t)result, execution_time);

    return execution_time;
}
