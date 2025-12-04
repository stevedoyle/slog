// iommu_domain_mmap.bpf.c
#include "vmlinux.h"
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>

struct map_event {
    __u32 pid;
    __u64 domain_ptr;      // iommu_domain pointer
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

// Trace iommu_map - the actual domain mapping function
SEC("kprobe/iommu_map")
int trace_iommu_map(struct pt_regs *ctx)
{
    struct map_event *e = bpf_ringbuf_reserve(&events, sizeof(*e), 0);
    if (!e)
        return 0;
    
    // iommu_map(struct iommu_domain *domain, unsigned long iova,
    //           phys_addr_t paddr, size_t size, int prot, gfp_t gfp)
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

// Trace iommu_unmap
SEC("kprobe/iommu_unmap")
int trace_iommu_unmap(struct pt_regs *ctx)
{
    struct map_event *e = bpf_ringbuf_reserve(&events, sizeof(*e), 0);
    if (!e)
        return 0;
    
    // iommu_unmap(struct iommu_domain *domain, unsigned long iova, size_t size)
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
