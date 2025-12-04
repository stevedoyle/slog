#include "vmlinux.h"
#include <bpf/bpf_helpers.h>

SEC("tp/syscalls/sys_enter_execve")
int handle_execve(struct trace_event_raw_sys_enter *ctx) {
    // Get the filename argument (first argument of execve)
    const char *filename = (const char *)ctx->args[0];

    // Log the filename (for demonstration purposes, we use bpf_printk)
    bpf_printk("execve called with filename: %s\n", filename);

    return 0;
}

char LICENSE[] SEC("license") = "GPL";
