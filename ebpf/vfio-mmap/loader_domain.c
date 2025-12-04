// loader_domain.c
#include <stdio.h>
#include <bpf/libbpf.h>
#include <bpf/bpf.h>
#include <linux/types.h>

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

static int handle_event(void *ctx, void *data, size_t len)
{
    struct map_event *e = data;
    printf("%s: pid=%d comm=%s domain=0x%llx iova=0x%llx->0x%llx paddr=0x%llx size=0x%llx prot=0x%x\n",
           e->is_map ? "MAP  " : "UNMAP",
           e->pid, e->comm, e->domain_ptr,
           e->iova, e->iova + e->size, e->paddr, e->size, e->prot);
    return 0;
}

int main()
{
    struct bpf_object *obj;
    struct bpf_program *prog;
    struct bpf_link *link_map = NULL;
    struct bpf_link *link_unmap = NULL;
    int err;

    obj = bpf_object__open_file("iommu_domain_mmap.bpf.o", NULL);
    if (!obj) {
        fprintf(stderr, "Failed to open BPF object\n");
        return 1;
    }

    err = bpf_object__load(obj);
    if (err) {
        fprintf(stderr, "Failed to load BPF object: %d\n", err);
        return 1;
    }

    // Attach iommu_map probe
    prog = bpf_object__find_program_by_name(obj, "trace_iommu_map");
    if (!prog) {
        fprintf(stderr, "Failed to find trace_iommu_map program\n");
        return 1;
    }
    link_map = bpf_program__attach(prog);
    if (!link_map) {
        fprintf(stderr, "Failed to attach trace_iommu_map\n");
        return 1;
    }

    // Attach iommu_unmap probe
    prog = bpf_object__find_program_by_name(obj, "trace_iommu_unmap");
    if (!prog) {
        fprintf(stderr, "Failed to find trace_iommu_unmap program\n");
        return 1;
    }
    link_unmap = bpf_program__attach(prog);
    if (!link_unmap) {
        fprintf(stderr, "Failed to attach trace_iommu_unmap\n");
        return 1;
    }

    struct ring_buffer *rb = ring_buffer__new(
        bpf_map__fd(bpf_object__find_map_by_name(obj, "events")),
        handle_event, NULL, NULL);
    if (!rb) {
        fprintf(stderr, "Failed to create ring buffer\n");
        return 1;
    }

    printf("Tracing IOMMU domain mappings... Press Ctrl+C to exit\n");

    while (ring_buffer__poll(rb, -1) >= 0);

    ring_buffer__free(rb);
    bpf_link__destroy(link_map);
    bpf_link__destroy(link_unmap);
    bpf_object__close(obj);

    return 0;
}
