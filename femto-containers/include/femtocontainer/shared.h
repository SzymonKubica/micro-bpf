/*
 * Copyright (C) 2021 Inria
 * Copyright (C) 2021 Koen Zandberg <koen@bergzand.net>
 *
 * This file is subject to the terms and conditions of the GNU Lesser
 * General Public License v2.1. See the file LICENSE in the top level
 * directory for more details.
 */

#ifndef FEMTOCONTAINER_SHARED_H
#define FEMTOCONTAINER_SHARED_H

#ifdef __cplusplus
extern "C" {
#endif

#define __bpf_shared_ptr(type, name)    \
union {                 \
    type name;          \
    uint64_t :64;          \
} __attribute__((aligned(8)))


#ifdef __cplusplus
}
#endif
#endif /* FEMTOCONTAINER_SHARED_H */
