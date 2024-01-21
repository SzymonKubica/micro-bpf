/*
 * Copyright (C) 2020 Inria
 * Copyright (C) 2020 Koen Zandberg <koen@bergzand.net>
 *
 * This file is subject to the terms and conditions of the GNU Lesser
 * General Public License v2.1. See the file LICENSE in the top level
 * directory for more details.
 */

#include <stdint.h>
// Added for the debug printf.
#include <stdio.h>
#include <stdbool.h>
#include "assert.h"

#include "femtocontainer/femtocontainer.h"
#include "femtocontainer/instruction.h"
#include "femtocontainer/config.h"

extern int f12r_run(f12r_t *femtoc, const void *ctx, int64_t *result);

static int _execute(f12r_t *femtoc, void *ctx, int64_t *result)
{
    assert(femtoc->flags & FC_FLAG_SETUP_DONE);
    return f12r_run(femtoc, ctx, result);
}

int f12r_execute(f12r_t *femtoc, void *ctx, size_t ctx_len, int64_t *result)
{
    (void)ctx;
    (void)ctx_len;
    femtoc->arg_region.start = NULL;
    femtoc->arg_region.len = 0;

    return _execute(femtoc, ctx, result);
}

int f12r_execute_ctx(f12r_t *femtoc, void *ctx, size_t ctx_len, int64_t *result)
{
    printf("[BPF VM]: Executing f12r VM\n");
    femtoc->arg_region.start = ctx;
    femtoc->arg_region.len = ctx_len;
    femtoc->arg_region.flag = (FC_MEM_REGION_READ | FC_MEM_REGION_WRITE);

    return _execute(femtoc, ctx, result);
}

void f12r_setup(f12r_t *femtoc)
{
    femtoc->stack_region.start = femtoc->stack;
    femtoc->stack_region.len = femtoc->stack_size;
    femtoc->stack_region.flag = (FC_MEM_REGION_READ | FC_MEM_REGION_WRITE);
    femtoc->stack_region.next = &femtoc->data_region;

    femtoc->data_region.start = f12r_data(femtoc);
    femtoc->data_region.len = f12r_header(femtoc)->data_len;
    femtoc->data_region.flag = (FC_MEM_REGION_READ | FC_MEM_REGION_WRITE);
    femtoc->data_region.next = &femtoc->rodata_region;

    femtoc->rodata_region.start = f12r_rodata(femtoc);
    femtoc->rodata_region.len = f12r_header(femtoc)->rodata_len;
    femtoc->rodata_region.flag = FC_MEM_REGION_READ;
    femtoc->rodata_region.next = &femtoc->arg_region;

    femtoc->arg_region.next = NULL;
    femtoc->arg_region.start = NULL;
    femtoc->arg_region.len = 0;

    femtoc->flags |= FC_FLAG_SETUP_DONE;
}

void f12r_add_region(f12r_t *femtoc, f12r_mem_region_t *region,
                    void *start, size_t len, uint8_t flags)
{
    region->next = femtoc->arg_region.next;
    region->start = start;
    region->len = len;
    region->flag = flags;
    femtoc->arg_region.next = region;
}
