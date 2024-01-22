/*
 * Copyright (C) 2019 Kaspar Schleiser <kaspar@schleiser.de>
 *
 * This file is subject to the terms and conditions of the GNU Lesser
 * General Public License v2.1. See the file LICENSE in the top level
 * directory for more details.
 */

#include "femtocontainer/femtocontainer.h"
#include "femtocontainer/shared.h"
#include "fmt.h"
#include "net/gcoap.h"
#include "net/nanocoap.h"
#include "suit/storage.h"
#include "suit/storage/ram.h"
#include "suit/transport/coap.h"
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define GCOAP_BPF_APP_SIZE 2048
static uint8_t _stack[512] = {0};

/* Helper structs */
typedef struct {
    __bpf_shared_ptr(void *,
                     pkt); /**< Opaque pointer to the coap_pkt_t struct */
    __bpf_shared_ptr(uint8_t *, buf); /**< Packet buffer */
    size_t buf_len;                   /**< Packet buffer length */
} f12r_coap_ctx_t;

static ssize_t _riot_board_handler(coap_pkt_t *pkt, uint8_t *buf, size_t len,
                                   void *context)
{
    (void)context;
    return coap_reply_simple(pkt, COAP_CODE_205, buf, len, COAP_FORMAT_TEXT,
                             (uint8_t *)RIOT_BOARD, strlen(RIOT_BOARD));
}

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

static ssize_t _bpf_handler(coap_pkt_t *pdu, uint8_t *buf, size_t len,
                                  coap_request_ctx_t *ctx)
{
    char *location = ctx->resource->context;
    char reply[12] = {0};

    printf("[BPF handler]: getting appropriate SUIT backend depending on the "
           "storage "
           "location id. \n");

    // For now hard code the location to be .ram.0
    // TODO: fix later
    // location = ".ram.0";
    suit_storage_t *storage = suit_storage_find_by_id(location);

    assert(storage);

    printf("[BPF handler]: setting suit storage active location: %s\n",
           location);
    suit_storage_set_active_location(storage, location);
    const uint8_t *mem_region;
    size_t length;

    printf("[BPF handler]: getting a pointer to the data stored in the SUIT "
           "location. \n");
    suit_storage_read_ptr(storage, &mem_region, &length);

    printf("[BPF handler]: Application bytecode:\n");
    for (size_t i = 0; i < length; i++) {
        printf("%02x", mem_region[i]);
        // Add a new line every 8x8 bits -> each eBPF instruction is 64 bits
        // long.
        if (i % 8 == 7) {
            printf("\n");
        }
    }
    printf("\n");

    printf("[BPF handler]: initialising the eBPF application struct\n");
    _bpf.application = mem_region;
    _bpf.application_len = length;

    f12r_mem_region_t mem_pdu;
    f12r_mem_region_t mem_pkt;

    f12r_coap_ctx_t bpf_ctx = {
        .pkt = pdu,
        .buf = buf,
        .buf_len = len,
    };

    f12r_add_region(&_bpf, &mem_pdu, pdu->hdr, 256,
                    FC_MEM_REGION_READ | FC_MEM_REGION_WRITE);
    f12r_add_region(&_bpf, &mem_pkt, pdu, sizeof(coap_pkt_t),
                    FC_MEM_REGION_READ | FC_MEM_REGION_WRITE);

    f12r_setup(&_bpf);
    int64_t result = -1;
    printf("[BPF handler]: executing VM\n");
    int res = f12r_execute_ctx(&_bpf, &bpf_ctx, sizeof(bpf_ctx), &result);

    size_t reply_len = fmt_s32_dfp(reply, result, 0);

    printf("[BPF handler]: Execution complete res=%i, result=%i\n", res,
           (int)result);
    return coap_reply_simple(pdu, COAP_CODE_204, buf, len, 0, (uint8_t *)reply,
                             reply_len);
}

static ssize_t _firmware_pull_handler(coap_pkt_t *pdu, uint8_t *buf, size_t len,
                                      void *ctx)
{

    char *suit_arg =
        "coap://[fe80::87d2:5291:a8ea:23f2%5]/suit_manifest.signed";
    suit_worker_trigger(suit_arg, strlen(suit_arg));

    return coap_reply_simple(pdu, COAP_CODE_204, buf, len, 0, 0, 1);
}

/* must be sorted by path (ASCII order) */
const coap_resource_t coap_resources[] = {
    COAP_WELL_KNOWN_CORE_DEFAULT_HANDLER,
    {"/bpf/exec/0", COAP_POST, _bpf_handler, ".ram.0"},
    {"/bpf/exec/1", COAP_POST, _bpf_handler, ".ram.1"},
    {"/riot/board", COAP_GET, _riot_board_handler, NULL},
    {"/pull", COAP_GET, _firmware_pull_handler, NULL},

    /* this line adds the whole "/suit"-subtree */
    SUIT_COAP_SUBTREE,
};

const unsigned coap_resources_numof = ARRAY_SIZE(coap_resources);
