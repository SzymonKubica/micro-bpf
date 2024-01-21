/*
 * Copyright (C) 2021 Inria
 * Copyright (C) 2021 Koen Zandberg <koen@bergzand.net>
 *
 * This file is subject to the terms and conditions of the GNU Lesser
 * General Public License v2.1. See the file LICENSE in the top level
 * directory for more details.
 */

#ifndef FEMTOCONTAINER_CONFIG_H
#define FEMTOCONTAINER_CONFIG_H

#ifndef FEMTO_CONTAINER_ENABLE_ALU32
#define FEMTO_CONTAINER_ENABLE_ALU32 (0)
#endif

#ifndef FEMTO_CONTAINER_BRANCHES_ALLOWED
#define FEMTO_CONTAINER_BRANCHES_ALLOWED 200
#endif


#ifndef FEMTO_CONTAINER_EXTERNAL_CALLS
static inline f12r_call_t f12r_get_external_call(uint32_t num)
{
    (void)num;
    return NULL;
}
#else
f12r_call_t f12r_get_external_call(uint32_t num);
#endif

#endif /* FEMTOCONTAINER_CONFIG_H */
