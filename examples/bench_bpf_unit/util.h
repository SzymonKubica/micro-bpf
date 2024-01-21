/*
 * Copyright (C) 2020 Inria
 * Copyright (C) 2020 Koen Zandberg <koen@bergzand.net>
 *
 * This file is subject to the terms and conditions of the GNU Lesser
 * General Public License v2.1. See the file LICENSE in the top level
 * directory for more details.
 */

#ifndef TEST_UTIL_H
#define TEST_UTIL_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#include "bpf/instruction.h"
#if BPF_COQ
typedef struct __attribute__((packed)) {
    uint32_t magic;      /**< Magic number */
    uint32_t version;    /**< Version of the application */
    uint32_t flags;
    uint32_t data_len;   /**< Length of the data section */
    uint32_t rodata_len; /**< Length of the rodata section */
    uint32_t text_len;   /**< Length of the text section */
    uint32_t functions;  /**< Number of functions available */
} rbpf_header_t;
#elif FEMTO
#define rbpf_header_t f12r_header_t
#define BPF_FLAG_PREFLIGHT_DONE FC_FLAG_PREFLIGHT_DONE
#else
#include "bpf.h"
#endif

typedef struct {
    bpf_instruction_t instruction;
    const char *name;
} test_content_t;

typedef struct {
    rbpf_header_t header;
    uint8_t rodata[68];
    uint64_t text[NUM_INSTRUCTIONS + 1];
} test_application_t;

void fill_instruction(const bpf_instruction_t *instr, test_application_t *test_app);

#ifdef __cplusplus
}
#endif
#endif /* TEST_UTIL_H */
/** @} */

