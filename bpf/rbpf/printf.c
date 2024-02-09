#include <linux/bpf.h>
#include <linux/in.h>
#include <linux/ip.h>
#include <linux/tcp.h>
#include <stdint.h>
#include <string.h>
#include <bpf/bpf_helpers.h>
#include "helpers.h"


SEC(".main")
int fletcher_32(struct __sk_buff *skb)
{
    // The format string cannot be passed in directly as we need a pointer to
    // it, so we have to declar a char array explicitly
    char fmt[] = "Time now in ms: %d\n";

    uint32_t now = bpf_now_ms();
    bpf_printf(fmt, now, 0, 0);
    return now;
}

