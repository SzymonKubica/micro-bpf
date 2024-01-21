/*
 * Copyright (C) 2021 Inria
 * Copyright (C) 2021 Koen Zandberg <koen@bergzand.net>
 *
 * This file is subject to the terms and conditions of the GNU Lesser
 * General Public License v2.1. See the file LICENSE in the top level
 * directory for more details.
 */

#ifndef FEMTOCONTAINER_H
#define FEMTOCONTAINER_H

#include <stdint.h>
#include <stdlib.h>

#ifdef __cplusplus
extern "C" {
#endif
#define FC_STACK_SIZE  512

#define RBPF_MAGIC_NO 0x72425046 /**< Magic header number: "rBPF" */

typedef struct __attribute__((packed)) {
    uint32_t magic;      /**< Magic number */
    uint32_t version;    /**< Version of the application */
    uint32_t flags;
    uint32_t data_len;   /**< Length of the data section */
    uint32_t rodata_len; /**< Length of the rodata section */
    uint32_t text_len;   /**< Length of the text section */
    uint32_t functions;  /**< Number of functions available */
} f12r_header_t;

typedef struct __attribute__((packed)) {
    uint16_t name_offset; /**< Offset in the rodata for the name */
    uint16_t flags;       /**< Flags for this function */
    uint16_t location_offset; /**< Location in the text section where the function starts */
} f12r_function_t;

typedef enum {
    FC_POLICY_CONTINUE,            /**< Always execute next hook */
    FC_POLICY_ABORT_ON_NEGATIVE,   /**< Execute next script unless result is negative */
    FC_POLICY_ABORT_ON_POSITIVE,   /**< Execute next script unless result is non-zero positive */
    FC_POLICY_SINGLE,              /**< Always stop after this execution */
} f12r_hook_policy_t;

enum {
    FC_OK = 0,
    FC_ILLEGAL_INSTRUCTION = -1,
    FC_ILLEGAL_MEM         = -2,
    FC_ILLEGAL_JUMP        = -3,
    FC_ILLEGAL_CALL        = -4,
    FC_ILLEGAL_LEN         = -5,
    FC_ILLEGAL_REGISTER    = -6,
    FC_NO_RETURN           = -7,
    FC_OUT_OF_BRANCHES     = -8,
    FC_ILLEGAL_DIV         = -9,
};

typedef struct f12r_mem_region f12r_mem_region_t;

#define FC_MEM_REGION_READ     0x01
#define FC_MEM_REGION_WRITE    0x02
#define FC_MEM_REGION_EXEC     0x04

/**
 * @brief Femto-Container memory region
 */
struct f12r_mem_region {
    f12r_mem_region_t *next;
    const uint8_t *start;
    size_t len;
    uint8_t flag;
};

#define FC_FLAG_SETUP_DONE        0x01
#define FC_FLAG_PREFLIGHT_DONE    0x02
#define FC_CONFIG_NO_RETURN       0x0100 /**< Script doesn't need to have a return */

typedef struct {
    f12r_mem_region_t stack_region;
    f12r_mem_region_t rodata_region;
    f12r_mem_region_t data_region;
    f12r_mem_region_t arg_region;
    const uint8_t *application; /**< Application bytecode */
    size_t application_len;     /**< Application length */
    uint8_t *stack;             /**< VM stack, must be a multiple of 8 bytes and aligned */
    size_t stack_size;          /**< VM stack size in bytes */
    uint16_t flags;
    uint32_t branches_remaining; /**< Number of allowed branch instructions remaining */
} f12r_t;

typedef struct f12r_hook f12r_hook_t;

struct bpf_hook {
    struct f12r_hook *next;
    f12r_t *application;
    uint32_t executions;
    f12r_hook_policy_t policy;
};

typedef uint32_t (*f12r_call_t)(f12r_t *fc, uint64_t *regs);

void f12r_init(void);
void f12r_setup(f12r_t *femtoc);

int f12r_verify_preflight(f12r_t *femtoc);

int f12r_execute(f12r_t *femtoc, void *ctx, size_t ctx_size, int64_t *result);
int f12r_execute_ctx(f12r_t *femtoc, void *ctx, size_t ctx_size, int64_t *result);
//int f12r_hook_execute(f12r_hook_trigger_t trigger, void *ctx, size_t ctx_size, int64_t *script_res);
//int f12r_hook_install(f12r_hook_t *hook, f12r_hook_trigger_t trigger);

int f12r_install_hook(f12r_t *femtoc);

void f12r_add_region(f12r_t *femtoc, f12r_mem_region_t *region,
                    void *start, size_t len, uint8_t flags);

int f12r_store_allowed(const f12r_t *femtoc, void *addr, size_t size);
int f12r_load_allowed(const f12r_t *femtoc, void *addr, size_t size);

static inline f12r_header_t *f12r_header(const f12r_t *femtoc)
{
    return (f12r_header_t*)femtoc->application;
}

static inline void *f12r_rodata(const f12r_t *femtoc)
{
    f12r_header_t *header = f12r_header(femtoc);
    return (uint8_t*)header + sizeof(f12r_t) + header->data_len;
}

static inline void *f12r_data(const f12r_t *femtoc)
{
    f12r_header_t *header = f12r_header(femtoc);
    return (uint8_t*)header + sizeof(f12r_t);
}

static inline void *f12r_text(const f12r_t *femtoc)
{
    f12r_header_t *header = f12r_header(femtoc);
    return (uint8_t*)header + sizeof(f12r_header_t) + header->data_len + header->rodata_len;
}

static inline size_t f12r_text_len(const f12r_t *femtoc)
{
    f12r_header_t *header = f12r_header(femtoc);
    return header->text_len;
}

/* to be implemented by platform specifc code. */
void f12r_store_init(void);

#ifdef __cplusplus
}
#endif
#endif /* FEMTOCONTAINER_H */
