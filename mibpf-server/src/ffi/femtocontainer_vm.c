#include "femtocontainer/femtocontainer.h"
#include "femtocontainer/shared.h"
#include "fmt.h"
#include "log.h"
#include "suit/storage.h"
#include "suit/storage/ram.h"
#include "suit/transport/coap.h"
#include <stdint.h>
#include <stdlib.h>

static f12r_t _bpf = {
    .stack_region = NULL,
    .rodata_region = NULL,
    .data_region = NULL,
    .arg_region = NULL,
    .application = NULL,  /**< Application bytecode */
    .application_len = 0, /**< Application length */
    .stack = NULL,        /** < We set the stack to null and enforce that the
                                caller of the methods needs to pass the stack in */
    .stack_size = 512,    // In line with the eBPF specification
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

typedef struct {
    coap_pkt_t *pdu;
    uint8_t *buf;
    size_t len;
} pkt_buf;

uint32_t verify_fc_program(uint8_t *program, uint32_t program_len)
{

    LOG_DEBUG("[BPF handler]: verifying the eBPF program\n");
    _bpf.application = program;
    _bpf.application_len = program_len;
    // The verification should have already been done
    _bpf.flags = FC_CONFIG_NO_RETURN;
    LOG_DEBUG("Program address: %p\n", program);

    LOG_DEBUG("[BPF]: executing gcoap handler\n");

    f12r_setup(&_bpf);
    return f12r_verify_preflight(&_bpf);
}

void initialize_fc_vm(uint8_t *program, uint32_t program_len)
{
    LOG_DEBUG("[BPF handler]: initialising the eBPF application struct\n");
    _bpf.application = program;
    _bpf.application_len = program_len;
    // The verification should have already been done
    _bpf.flags |= FC_FLAG_PREFLIGHT_DONE;
    f12r_setup(&_bpf);
}

uint32_t execute_fc_vm(uint8_t *stack, int64_t *result)
{
    _bpf.stack = stack;
    return f12r_execute(&_bpf, 0, 64, result);
}

uint32_t execute_fc_vm_on_coap_pkt(uint8_t *stack, pkt_buf *ctx,
                                   int64_t *result)
{

    coap_pkt_t *pdu = ctx->pdu;
    uint8_t *buf = ctx->buf;
    size_t len = ctx->len;

    f12r_mem_region_t mem_pdu;
    f12r_mem_region_t mem_pkt;
    f12r_mem_region_t mem_buff;

    f12r_coap_ctx_t bpf_ctx = {
        .pkt = pdu,
        .buf = buf,
        .buf_len = len,
    };

    f12r_add_region(&_bpf, &mem_pdu, pdu->hdr, 256,
                    FC_MEM_REGION_READ | FC_MEM_REGION_WRITE);
    f12r_add_region(&_bpf, &mem_pkt, pdu, sizeof(coap_pkt_t),
                    FC_MEM_REGION_READ | FC_MEM_REGION_WRITE);
    // Allow for reading and writing to the whole packet payload,
    f12r_add_region(&_bpf, &mem_buff, pdu->payload, 512,
                    FC_MEM_REGION_READ | FC_MEM_REGION_WRITE);

    _bpf.stack = stack;
    return f12r_execute_ctx(&_bpf, &bpf_ctx, 64, result);
}
