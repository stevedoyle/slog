// iommu_domain_mmap.bpf.c
#include "vmlinux.h"
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>

struct map_event {
    __u32 pid;
    __u64 domain_ptr;
    __u64 iova;
    __u64 paddr;
    __u64 size;
    __u32 prot;
    char comm[16];
    __u8 is_map;
};

struct {
    __uint(type, BPF_MAP_TYPE_RINGBUF);
    __uint(max_entries, 256 * 1024);
} events SEC(".maps");

// Config map to pass target process name from userspace
struct {
    __uint(type, BPF_MAP_TYPE_ARRAY);
    __uint(max_entries, 1);
    __type(key, __u32);
    __type(value, char[16]);
} target_comm SEC(".maps");

static inline int is_target_process(void)
{
    char comm[16];
    bpf_get_current_comm(&comm, sizeof(comm));
    
    __u32 key = 0;
    char *target = bpf_map_lookup_elem(&target_comm, &key);
    if (!target)
        return 0;  // No filter set, capture nothing
    
    // Compare comm with target
    for (int i = 0; i < 16; i++) {
        if (target[i] == '\0')
            return 1;  // Matched up to null terminator
        if (comm[i] != target[i])
            return 0;
    }
    return 1;
}

SEC("kprobe/iommu_map")
int trace_iommu_map(struct pt_regs *ctx)
{
    if (!is_target_process())
        return 0;
    
    struct map_event *e = bpf_ringbuf_reserve(&events, sizeof(*e), 0);
    if (!e)
        return 0;
    
    void *domain = (void *)PT_REGS_PARM1(ctx);
    unsigned long iova = (unsigned long)PT_REGS_PARM2(ctx);
    unsigned long paddr = (unsigned long)PT_REGS_PARM3(ctx);
    unsigned long size = (unsigned long)PT_REGS_PARM4(ctx);
    int prot = (int)PT_REGS_PARM5(ctx);
    
    e->pid = bpf_get_current_pid_tgid() >> 32;
    bpf_get_current_comm(&e->comm, sizeof(e->comm));
    e->domain_ptr = (__u64)domain;
    e->iova = iova;
    e->paddr = paddr;
    e->size = size;
    e->prot = prot;
    e->is_map = 1;
    
    bpf_ringbuf_submit(e, 0);
    return 0;
}

SEC("kprobe/iommu_unmap")
int trace_iommu_unmap(struct pt_regs *ctx)
{
    if (!is_target_process())
        return 0;
    
    struct map_event *e = bpf_ringbuf_reserve(&events, sizeof(*e), 0);
    if (!e)
        return 0;
    
    void *domain = (void *)PT_REGS_PARM1(ctx);
    unsigned long iova = (unsigned long)PT_REGS_PARM2(ctx);
    unsigned long size = (unsigned long)PT_REGS_PARM3(ctx);
    
    e->pid = bpf_get_current_pid_tgid() >> 32;
    bpf_get_current_comm(&e->comm, sizeof(e->comm));
    e->domain_ptr = (__u64)domain;
    e->iova = iova;
    e->paddr = 0;
    e->size = size;
    e->prot = 0;
    e->is_map = 0;
    
    bpf_ringbuf_submit(e, 0);
    return 0;
}

char LICENSE[] SEC("license") = "GPL";
