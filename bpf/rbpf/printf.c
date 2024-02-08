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
    // Testing update

    uint32_t now = bpf_now_ms();
    bpf_trace_printk("", 20, now);
    bpf_printf("%d", now);
    return 0;
}

