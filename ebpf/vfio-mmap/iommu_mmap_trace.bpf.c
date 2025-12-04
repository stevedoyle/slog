// iommu_mmap_trace.bpf.c
#include "vmlinux.h"
#include <bpf/bpf_helpers.h>
#include <bpf/bpf_tracing.h>

struct mmap_event {
    __u32 pid;
    __u64 iova;
    __u64 size;
    __u64 vaddr;
    __u32 flags;
    char comm[16];
    __u8 is_map;  // 1=map, 0=unmap
};

struct {
    __uint(type, BPF_MAP_TYPE_RINGBUF);
    __uint(max_entries, 256 * 1024);
} events SEC(".maps");

// Trace VFIO_IOMMU_MAP_DMA
SEC("kprobe/vfio_iommu_type1_ioctl")
int trace_vfio_ioctl(struct pt_regs *ctx)
{
    unsigned int cmd = (unsigned int)PT_REGS_PARM2(ctx);
    unsigned long arg = (unsigned long)PT_REGS_PARM3(ctx);
    
    // VFIO_IOMMU_MAP_DMA = 0x3b71
    // VFIO_IOMMU_UNMAP_DMA = 0x3b72
    if (cmd != 0x3b71 && cmd != 0x3b72)
        return 0;
    
    struct mmap_event *e = bpf_ringbuf_reserve(&events, sizeof(*e), 0);
    if (!e)
        return 0;
    
    e->pid = bpf_get_current_pid_tgid() >> 32;
    bpf_get_current_comm(&e->comm, sizeof(e->comm));
    e->is_map = (cmd == 0x3b71) ? 1 : 0;
    
    // Read vfio_iommu_type1_dma_map struct from userspace
    struct {
        __u32 argsz;
        __u32 flags;
        __u64 vaddr;
        __u64 iova;
        __u64 size;
    } dma_map;
    
    bpf_probe_read_user(&dma_map, sizeof(dma_map), (void *)arg);
    
    e->iova = dma_map.iova;
    e->size = dma_map.size;
    e->vaddr = dma_map.vaddr;
    e->flags = dma_map.flags;
    
    bpf_ringbuf_submit(e, 0);
    return 0;
}

char LICENSE[] SEC("license") = "GPL";
