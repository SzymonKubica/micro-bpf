/*
 * Copyright (C) 2021 Inria
 * Copyright (C) 2021 Koen Zandberg <koen@bergzand.net>
 *
 * This file is subject to the terms and conditions of the GNU Lesser
 * General Public License v2.1. See the file LICENSE in the top level
 * directory for more details.
 */

#ifndef FEMTOCONTAINER_BUILTIN_CALLS_H
#define FEMTOCONTAINER_BUILTIN_CALLS_H

#ifdef __cplusplus
extern "C" {
#endif

uint32_t bpf_vm_store_local(f12r_t *bpf, uint32_t key, uint32_t value, uint32_t a3, uint32_t a4, uint32_t a5);
uint32_t bpf_vm_store_global(f12r_t *bpf, uint32_t key, uint32_t value, uint32_t a3, uint32_t a4, uint32_t a5);
uint32_t bpf_vm_fetch_local(f12r_t *bpf, uint32_t key, uint32_t value, uint32_t a3, uint32_t a4, uint32_t a5);
uint32_t bpf_vm_fetch_global(f12r_t *bpf, uint32_t key, uint32_t value, uint32_t a3, uint32_t a4, uint32_t a5);

#ifdef __cplusplus
}
#endif
#endif /* FEMTOCONTAINER_BUILTIN_CALLS_H */
