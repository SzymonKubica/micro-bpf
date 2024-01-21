/*
 * Copyright (C) 2021 Inria
 * Copyright (C) 2021 Koen Zandberg <koen@bergzand.net>
 *
 * This file is subject to the terms and conditions of the GNU Lesser
 * General Public License v2.1. See the file LICENSE in the top level
 * directory for more details.
 */

#include <stdint.h>
#include <stdlib.h>
#include <stdbool.h>

#include "femtocontainer/femtocontainer.h"
#include "femtocontainer/builtin_shared.h"
#include "femtocontainer/instruction.h"
#include "femtocontainer/config.h"

static bool _f12r_check_call(uint32_t num)
{
    switch(num) {
        default:
            return f12r_get_external_call(num) ? true : false;
    }
}


int f12r_verify_preflight(f12r_t *femtoc)
{

    const bpf_instruction_t *application = f12r_text(femtoc);
    size_t length = f12r_text_len(femtoc);
    if (femtoc->flags & FC_FLAG_PREFLIGHT_DONE) {
        return FC_OK;
    }

    if (length & 0x7) {
        return FC_ILLEGAL_LEN;
    }


    for (const bpf_instruction_t *i = application;
            i < (bpf_instruction_t*)((uint8_t*)application + length); i++) {
        /* Check if register values are valid */
        if (i->dst >= 11 || i->src >= 11) {
            return FC_ILLEGAL_REGISTER;
        }

        /* Double length instruction */
        if (i->opcode == 0x18) {
            i++;
            continue;
        }

        /* Only instruction-specific checks here */
        if ((i->opcode & BPF_INSTRUCTION_CLS_MASK) == BPF_INSTRUCTION_CLS_BRANCH) {
            intptr_t target = (intptr_t)(i + i->offset);
            /* Check if the jump target is within bounds. The address is
             * incremented after the jump by the regular PC increase */
            if ((target >= (intptr_t)((uint8_t*)application + length))
                || (target < (intptr_t)application)) {
                return FC_ILLEGAL_JUMP;
            }
        }

        if (i->opcode == (BPF_INSTRUCTION_BRANCH_CALL | BPF_INSTRUCTION_CLS_BRANCH)) {
            if (!_f12r_check_call(i->immediate)) {
                return FC_ILLEGAL_CALL;
            }
        }
    }

    size_t num_instructions = length/sizeof(bpf_instruction_t);

    /* Check if the last instruction is a return instruction */
    if (application[num_instructions - 1].opcode != 0x95 && !(femtoc->flags & FC_CONFIG_NO_RETURN)) {
        return FC_NO_RETURN;
    }
    femtoc->flags |= FC_FLAG_PREFLIGHT_DONE;
    return FC_OK;
}
