#include "log.h"
#include "suit/storage.h"
#include "suit/storage/ram.h"
#include "suit/transport/coap.h"

/// Responsible for reading the BPF application bytecode from the SUIT storage
/// @param[out] buff      Target buffer where the read program is written.
/// @param[in]  location  SUIT ram storage location from where the bytecode is
/// loaded.
uint32_t load_bytes_from_suit_storage(uint8_t *buff, char *location)
{

    LOG_DEBUG("[SUIT storage loader]: getting SUIT storage given id: %s. \n",
              location);

    suit_storage_t *storage = suit_storage_find_by_id(location);

    assert(storage);

    LOG_DEBUG(
        "[SUIT storage loader]: setting suit storage active location: %s\n",
        location);

    suit_storage_set_active_location(storage, location);
    const uint8_t *mem_region;
    size_t length;

    LOG_DEBUG("[SUIT storage loader]: getting a pointer to the data stored in "
              "the SUIT "
              "location: %s.\n",
              location);
    suit_storage_read_ptr(storage, &mem_region, &length);

    LOG_DEBUG("[SUIT storage loader]: Application bytecode:\n");
    for (size_t i = 0; i < length; i++) {
        LOG_DEBUG("%02x", mem_region[i]);
        // Add a new line every 8x8 bits -> each eBPF instruction is 64 bits
        // long.
        if (i % 8 == 7) {
            LOG_DEBUG("\n");
        }
        // Write the byte in the data region into the target buffer
        *(buff + i) = mem_region[i];
    }
    LOG_DEBUG("\n");
    return length;
}

void initiate_suit_fetch(char *address, char *signed_manifest_name)
{
    char suit_arg[70];
    sprintf(suit_arg, "coap://[%s%%5]/%s", address, signed_manifest_name);
    LOG_DEBUG("Triggering the SUIT worker to fetch from %s on %s\n", address,
              signed_manifest_name);
    suit_worker_trigger(suit_arg, strlen(suit_arg));
}
