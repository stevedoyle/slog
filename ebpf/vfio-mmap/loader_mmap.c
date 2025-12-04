// loader.c
#include <stdio.h>
#include <bpf/libbpf.h>
#include <bpf/bpf.h>
#include <linux/types.h>

struct mmap_event {
    __u32 pid;
    __u64 iova;
    __u64 size;
    __u64 vaddr;
    __u32 flags;
    char comm[16];
    __u8 is_map;  // 1=map, 0=unmap
};

static int handle_event(void *ctx, void *data, size_t len)
{
    struct mmap_event *e = data;
    printf("%s: pid=%d comm=%s iova=0x%llx size=0x%llx vaddr=0x%llx flags=0x%x\n",
           e->is_map ? "MAP" : "UNMAP",
           e->pid, e->comm, e->iova, e->size, e->vaddr, e->flags);
    return 0;
}

int main()
{
    struct bpf_object *obj;
    struct bpf_program *prog;
    struct bpf_link *link = NULL;
    int err;

    obj = bpf_object__open_file("iommu_mmap_trace.bpf.o", NULL);
    if (!obj) {
        fprintf(stderr, "Failed to open BPF object\n");
        return 1;
    }

    err = bpf_object__load(obj);
    if (err) {
        fprintf(stderr, "Failed to load BPF object: %d\n", err);
        return 1;
    }

    prog = bpf_object__find_program_by_name(obj, "trace_vfio_ioctl");
    if (!prog) {
        fprintf(stderr, "Failed to find BPF program\n");
        return 1;
    }

    link = bpf_program__attach(prog);
    if (!link) {
        fprintf(stderr, "Failed to attach BPF program\n");
        return 1;
    }

    struct ring_buffer *rb = ring_buffer__new(
        bpf_map__fd(bpf_object__find_map_by_name(obj, "events")),
        handle_event, NULL, NULL);
    if (!rb) {
        fprintf(stderr, "Failed to create ring buffer\n");
        return 1;
    }

    printf("Tracing VFIO IOMMU operations... Press Ctrl+C to exit\n");

    while (ring_buffer__poll(rb, -1) >= 0);

    ring_buffer__free(rb);
    bpf_link__destroy(link);
    bpf_object__close(obj);

    return 0;
}

