/*
 * Copyright (C) 2020 Inria
 * Copyright (C) 2020 Koen Zandberg <koen@bergzand.net>
 *
 * This file is subject to the terms and conditions of the GNU Lesser
 * General Public License v2.1. See the file LICENSE in the top level
 * directory for more details.
 */

#ifndef BPF_BPFAPI_HELPERS_H
#define BPF_BPFAPI_HELPERS_H

#include <stdint.h>
#include "shared.h"

typedef signed ssize_t;

// Macro allowing for printing formatted strings without having to separately
// declare the format char[]. The do-while is needed in case the macro is invoked
// after an if statement without braces.
#define print(format, ...)                                                     \
do {                                                                           \
    char fmt[] = format;                                                       \
    bpf_printf(fmt, __VA_ARGS__);                                              \
} while(0);

#define PHYDAT_DIM                  (3U)
typedef struct {
    int16_t val[PHYDAT_DIM];    /**< the 3 generic dimensions of data */
    uint8_t unit;               /**< the (physical) unit of the data */
    int8_t scale;               /**< the scale factor, 10^*scale* */
} phydat_t;


/**
 * Opaque dummy type saul registration
 */
typedef void bpf_saul_reg_t;

static void *(*bpf_printf)(const char *fmt, ...) = (void *) BPF_FUNC_BPF_PRINTF;
// Added this one for printing a single debug value.
static void *(*bpf_print_debug)(uint32_t value) = (void *) BPF_FUNC_BPF_PRINT_DEBUG;

static int (*bpf_store_global)(uint32_t key, uint32_t value) = (void *) BPF_FUNC_BPF_STORE_GLOBAL;
static int (*bpf_store_local)(uint32_t key, uint32_t value) = (void *) BPF_FUNC_BPF_STORE_LOCAL;
static int (*bpf_fetch_global)(uint32_t key, uint32_t *value) = (void *) BPF_FUNC_BPF_FETCH_GLOBAL;
static int (*bpf_fetch_local)(uint32_t key, uint32_t *value) = (void *) BPF_FUNC_BPF_FETCH_LOCAL;
static uint32_t (*bpf_now_ms)(void) = (void *) BPF_FUNC_BPF_NOW_MS;

/* STDLIB */
static void *(*bpf_memcpy)(void *dest, const void *src, size_t n) = (void *) BPF_FUNC_BPF_MEMCPY;

/* SAUL calls */
static bpf_saul_reg_t *(*bpf_saul_reg_find_nth)(int pos) = (void *) BPF_FUNC_BPF_SAUL_REG_FIND_NTH;
static bpf_saul_reg_t *(*bpf_saul_reg_find_type)(uint8_t type) = (void *) BPF_FUNC_BPF_SAUL_REG_FIND_TYPE;
static int (*bpf_saul_reg_read)(bpf_saul_reg_t *dev, phydat_t *data) = (void *) BPF_FUNC_BPF_SAUL_REG_READ;
static int (*bpf_saul_reg_write)(bpf_saul_reg_t *dev, phydat_t *data) = (void *) BPF_FUNC_BPF_SAUL_REG_WRITE;

/* CoAP calls */
static void (*bpf_gcoap_resp_init)(bpf_coap_ctx_t *ctx, unsigned resp_code) = (void *) BPF_FUNC_BPF_GCOAP_RESP_INIT;
static ssize_t (*bpf_coap_opt_finish)(bpf_coap_ctx_t *ctx, unsigned opt) = (void *) BPF_FUNC_BPF_COAP_OPT_FINISH;
static void (*bpf_coap_add_format)(bpf_coap_ctx_t *ctx, uint32_t format) = (void *) BPF_FUNC_BPF_COAP_ADD_FORMAT;
static uint8_t *(*bpf_coap_get_pdu)(bpf_coap_ctx_t *ctx) = (void *) BPF_FUNC_BPF_COAP_GET_PDU;

/* FMT calls */
static size_t (*bpf_fmt_s16_dfp)(char *out, int16_t val, int fp_digits) = (void *) BPF_FUNC_BPF_FMT_S16_DFP;
static size_t (*bpf_fmt_u32_dec)(char *out, uint32_t val) = (void *) BPF_FUNC_BPF_FMT_U32_DEC;

/* ZTIMER calls */
static uint32_t (*bpf_ztimer_now)(void) = (void *) BPF_FUNC_BPF_ZTIMER_NOW;
static void (*bpf_ztimer_periodic_wakeup)(uint32_t *last_wakeup, uint32_t period) = (void *) BPF_FUNC_BPF_ZTIMER_PERIODIC_WAKEUP;

/* GPIO calls */
static uint64_t (*bpf_gpio_read)(uint32_t port, uint32_t pin) = (void *) BPF_FUNC_GPIO_READ;
static void (*bpf_gpio_write)(uint32_t port, uint32_t pin, uint32_t val) = (void *) BPF_FUNC_GPIO_WRITE;

#endif /* BPF_APPLICATION_CALL_H */
