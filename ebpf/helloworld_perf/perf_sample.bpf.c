#include <linux/bpf.h>
#include <bpf/bpf_helpers.h>
#include <linux/types.h>

// Map to store collected data
struct {
    __uint(type, BPF_MAP_TYPE_HASH);
    __uint(max_entries, 10240);
    __type(key, __u32);
    __type(value, __u64);
} sample_counts SEC(".maps");

SEC("perf_event")
int sample_handler(struct bpf_perf_event_data *ctx) {
    __u32 pid = bpf_get_current_pid_tgid() >> 32;
    __u64 *count = bpf_map_lookup_elem(&sample_counts, &pid);
    
    if (count) {
        __sync_fetch_and_add(count, 1);
    } else {
        __u64 init_val = 1;
        bpf_map_update_elem(&sample_counts, &pid, &init_val, BPF_ANY);
    }
    
    return 0;
}

char LICENSE[] SEC("license") = "GPL";