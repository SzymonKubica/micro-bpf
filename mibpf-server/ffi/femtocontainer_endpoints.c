#include "femtocontainer/femtocontainer.h"
#include "femtocontainer/shared.h"
#include "fmt.h"
#include "log.h"
#include "suit/storage.h"
#include "suit/storage/ram.h"
#include "suit/transport/coap.h"
#include <stdint.h>
#include <stdlib.h>

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
    LOG_INFO("Starting Femto-Container VM execution.");
    ztimer_acquire(ZTIMER_USEC);
    ztimer_now_t start = ztimer_now(ZTIMER_USEC);
    // Figure out the size of the context
    int res = f12r_execute_ctx(&_bpf, bpf_ctx, 64, &result);
    ztimer_now_t end = ztimer_now(ZTIMER_USEC);
    uint32_t execution_time = end - start;
    *return_value = result;

    LOG_INFO("Program returned: %d (%x)\n", result, result);
    LOG_INFO("Exit code: %d\n", res);
    LOG_INFO("Execution time: %d [us]\n", execution_time);

    return execution_time;
}

typedef struct {
    coap_pkt_t *pdu;
    uint8_t *buf;
    size_t len;
} pkt_buf;

uint32_t execute_fc_vm(uint8_t *program, uint32_t program_len, uint64_t *return_value)
{

    LOG_DEBUG("[BPF handler]: initialising the eBPF application struct\n");
    _bpf.application = program;
    _bpf.application_len = program_len;
    LOG_DEBUG("Program address: %p\n", program);

    LOG_DEBUG("[BPF]: executing gcoap handler\n");

    f12r_setup(&_bpf);
    int64_t result = -1;
    LOG_INFO("[BPF handler]: executing VM\n");
    ztimer_acquire(ZTIMER_USEC);
    ztimer_now_t start = ztimer_now(ZTIMER_USEC);
    // Figure out the size of the context
    int res = f12r_execute(&_bpf, 0, 64, &result);
    ztimer_now_t end = ztimer_now(ZTIMER_USEC);
    uint32_t execution_time = end - start;
    *return_value = result;

    LOG_INFO("Program returned: %d (%x)\n", result, result);
    LOG_INFO("Exit code: %d\n", res);
    LOG_INFO("Execution time: %d [us]\n", execution_time);

    return execution_time;
}

uint32_t execute_fc_vm_on_coap_pkt(uint8_t *program, uint32_t program_len, pkt_buf *ctx,
                                   uint64_t *return_value)
{

    coap_pkt_t *pdu = ctx->pdu;
    uint8_t *buf = ctx->buf;
    size_t len = ctx->len;

    LOG_DEBUG("[BPF handler]: initialising the eBPF application struct\n");
    _bpf.application = program;
    _bpf.application_len = program_len;
    LOG_DEBUG("Program address: %p\n", program);

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
    LOG_INFO("[BPF handler]: executing VM\n");
    ztimer_acquire(ZTIMER_USEC);
    ztimer_now_t start = ztimer_now(ZTIMER_USEC);
    // Figure out the size of the context
    int res = f12r_execute_ctx(&_bpf, &bpf_ctx, 64, &result);
    ztimer_now_t end = ztimer_now(ZTIMER_USEC);
    uint32_t execution_time = end - start;
    *return_value = result;

    LOG_INFO("Program returned: %d (%x)\n", result, result);
    LOG_INFO("Exit code: %d\n", res);
    LOG_INFO("Execution time: %d [us]\n", execution_time);

    return execution_time;
}
