#include <stdio.h>
#include <unistd.h>
#include <sys/syscall.h>
#include <sys/ioctl.h>
#include <linux/perf_event.h>
#include <bpf/libbpf.h>
#include <bpf/bpf.h>
#include <stdint.h>

static int perf_event_open(struct perf_event_attr *attr, pid_t pid,
                           int cpu, int group_fd, unsigned long flags) {
    return syscall(__NR_perf_event_open, attr, pid, cpu, group_fd, flags);
}

int main() {
    struct bpf_object *obj;
    struct bpf_program *prog;
    int prog_fd, perf_fd, map_fd;
    int num_cpus = sysconf(_SC_NPROCESSORS_ONLN);
    
    // Load eBPF program
    obj = bpf_object__open_file("perf_sample.bpf.o", NULL);
    if (!obj) {
        fprintf(stderr, "Failed to open BPF object\n");
        return 1;
    }
    
    if (bpf_object__load(obj)) {
        fprintf(stderr, "Failed to load BPF object\n");
        return 1;
    }
    
    prog = bpf_object__find_program_by_name(obj, "sample_handler");
    prog_fd = bpf_program__fd(prog);
    
    // Create perf event (sample CPU cycles at 99Hz)
    struct perf_event_attr attr = {
        .type = PERF_TYPE_HARDWARE,
        .config = PERF_COUNT_HW_CPU_CYCLES,
        .sample_period = 1000000 / 99,  // ~99Hz
        .freq = 0,
    };
    
    // Attach to all CPUs
    printf("Attaching to %d CPUs...\n", num_cpus);
    for (int cpu = 0; cpu < num_cpus; cpu++) {
        perf_fd = perf_event_open(&attr, -1, cpu, -1, 0);
        if (perf_fd < 0) {
            perror("perf_event_open");
            continue;
        }
        
        // Attach eBPF program to perf event
        if (ioctl(perf_fd, PERF_EVENT_IOC_SET_BPF, prog_fd) < 0) {
            perror("ioctl PERF_EVENT_IOC_SET_BPF");
            close(perf_fd);
            continue;
        }
        
        // Enable the event
        ioctl(perf_fd, PERF_EVENT_IOC_ENABLE, 0);
    }
    
    printf("Sampling for 10 seconds...\n");
    sleep(10);
    
    // Read results from map
    map_fd = bpf_object__find_map_fd_by_name(obj, "sample_counts");
    uint32_t key, next_key;
    uint64_t value;
    
    printf("\nPID\t\tSamples\n");
    key = 0;
    while (bpf_map_get_next_key(map_fd, &key, &next_key) == 0) {
        bpf_map_lookup_elem(map_fd, &next_key, &value);
        printf("%u\t\t%lu\n", next_key, value);
        key = next_key;
    }
    
    bpf_object__close(obj);
    return 0;
}