#include "log.h"
#include "sched.h"
#include "suit/storage.h"
#include "suit/storage/ram.h"
#include "suit/transport/coap.h"

static kernel_pid_t UPDATE_REQUESTOR_PID;

/// Responsible for reading the BPF application bytecode from the SUIT storage
/// @param[out] buff      Target buffer where the read program is written.
/// @param[in]  location  SUIT ram storage location from where the bytecode is
/// loaded.
uint32_t load_bytes_from_suit_storage(uint8_t *buff, uint8_t *location_id)
{

    char *location = (char *)location_id;
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

void handle_suit_storage_erase(uint8_t *location_id)
{

    char *location = (char *)location_id;
    suit_storage_t *storage = suit_storage_find_by_id(location);
    assert(storage);

    LOG_DEBUG("[SUIT storage]: erasing storage location: %s\n", location);
    suit_storage_erase(storage);
}

/// Overridden callback that is invoked by the SUIT worker thread once the
/// download of the file has been completed. In this case we override because
/// the thread that initialised the SUIT pull process is waiting for a
/// confirmation message.
void suit_worker_done_cb(int res)
{
    if (res == 0) {
        LOG_INFO("suit_worker: update successful\n");
    } else {
        LOG_INFO("suit_worker: update failed, hdr invalid\n ");
    }
    // We notify the requestor no matter what result we get so that
    // they become unblocked.
    msg_t msg;
    msg.type = 0;
    msg.content.value = res;
    LOG_DEBUG("suit_worker: sending completion notification to thread with "
              "PID: %d\n ",
              UPDATE_REQUESTOR_PID);
    msg_send(&msg, UPDATE_REQUESTOR_PID);
}

void initiate_suit_fetch(char *address, int network_interface,
                         char *signed_manifest_name, kernel_pid_t requestor)
{

    // We store the information who initiated the SUIT update so that we
    // can notify them in the callback.
    UPDATE_REQUESTOR_PID = requestor;

    char suit_arg[70];
    sprintf(suit_arg, "coap://[%s%%%d]/%s", address, network_interface,
            signed_manifest_name);
    LOG_DEBUG("Triggering the SUIT worker to fetch from %s on %s\n", address,
              signed_manifest_name);
    suit_worker_trigger(suit_arg, strlen(suit_arg));
}
