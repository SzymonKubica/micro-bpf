#include <linux/bpf.h>
#include <linux/in.h>
#include <linux/ip.h>
#include <linux/tcp.h>
#include <stdint.h>
#include <string.h>
#include <bpf/bpf_helpers.h>
#include "../helpers.h"

/*
 * Discussion points: eBPF seems to be limited w.r.t storing strings on the
 * stack When I tried including the 360B long string in the function code
 * directly as a constant. There was an error with illegal memory accesses. It
 * could be because that string couldn't fit in the stack.
 *
 */

#define ETH_ALEN 6
#define ETH_P_IP 0x0008 /* htons(0x0800) */
#define TCP_HDR_LEN 20

struct eth_hdr {
    unsigned char h_dest[ETH_ALEN];
    unsigned char h_source[ETH_ALEN];
    unsigned short h_proto;
};

SEC(".main")
int fletcher_32(struct __sk_buff *skb)
{

    uint32_t start = bpf_ztimer_now();
    void *data = (void *)(long)skb->data;
    void *data_end = (void *)(long)skb->data_end;
    struct eth_hdr *eth = data;
    struct iphdr *iph = data + sizeof(*eth);
    struct tcphdr *tcp = data + sizeof(*eth) + sizeof(*iph);

    // Ensure that there is some data
    if (data + sizeof(*eth) + sizeof(*iph) + sizeof(*tcp) > data_end)
        return -1;

    // After the tcp header, the packet data section begins which in our
    // case contains the payload with the string that is used to compute
    // the fletcher32 checksum.
    uint8_t *payload = data + sizeof(*eth) + sizeof(*iph) + sizeof(*tcp);

    // First extract the length of the message.
    uint32_t length = *(uint32_t *)payload;
    // Skip the remaining bits of the length field
    payload = (uint8_t *)payload + 4;

    uint16_t *data_ptr = (uint16_t *)payload;

    size_t len = (length + 1) & ~1; /* Round up len to words */
    uint32_t c0 = 0;
    uint32_t c1 = 0;
    uint32_t end = bpf_ztimer_now();

    print("Packet pre-processing time: %d [us]\n", end - start);

    start = bpf_ztimer_now();
    for (c0 = c1 = 0; len > 0;) {
        size_t blocklen = len;
        if (blocklen > 360 * 2) {
            blocklen = 360 * 2;
        }
        len -= blocklen;
        do {
            c0 = c0 + *data_ptr++;
            c1 = c1 + c0;
        } while ((blocklen -= 2));
        c0 = c0 % 65535;
        c1 = c1 % 65535;
    }

    uint32_t checksum = (c1 << 16 | c0);
    end = bpf_ztimer_now();

    print("Fletcher32 execution time: %d [us]\n", end - start);

    return checksum;
}
