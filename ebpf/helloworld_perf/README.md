
# eBPF CPU Profiler

A lightweight CPU profiler using eBPF and perf events to sample which processes are consuming CPU cycles.

## Overview

This program demonstrates how to use eBPF with perf events to perform CPU profiling. It attaches an eBPF program to hardware perf events that sample CPU cycles at approximately 99Hz, tracking which processes (by PID) are active on the CPU.

## Components

- **perf_sample.bpf.c**: The eBPF kernel-space program that runs on each perf event and collects sample counts per PID
- **perf_sample.c**: The userspace loader that sets up perf events, loads the eBPF program, and displays results
- **perf_sample.bpf.o**: Compiled eBPF bytecode (generated during build)

## How It Works

1. **Setup**: The userspace program creates perf events configured to sample `PERF_COUNT_HW_CPU_CYCLES` at ~99Hz on all CPUs
2. **Sampling**: Each time a CPU cycle sample occurs, the eBPF `sample_handler` function is triggered
3. **Tracking**: The handler extracts the current process ID and increments a counter in a BPF hash map
4. **Reporting**: After 10 seconds, the program reads the map and displays which PIDs received how many samples

Higher sample counts indicate processes that consumed more CPU time during the sampling period.

## Prerequisites

- Linux kernel with eBPF support (4.15+)
- clang (for compiling eBPF programs)
- gcc
- libbpf development files
- Root privileges (for loading eBPF programs and accessing perf events)

## Building

```bash
# Compile the eBPF program
clang -O2 -g -target bpf -D__TARGET_ARCH_x86 -c perf_sample.bpf.c -o perf_sample.bpf.o

# Compile the userspace program
gcc -o perf_sample perf_sample.c -lbpf
```

Or build both in one command:
```bash
clang -O2 -g -target bpf -D__TARGET_ARCH_x86 -c perf_sample.bpf.c -o perf_sample.bpf.o && gcc -o perf_sample perf_sample.c -lbpf
```

## Running

```bash
sudo ./perf_sample
```

The program will:
1. Load the eBPF program
2. Attach to all CPU cores
3. Sample for 10 seconds
4. Display a table of PIDs and their sample counts

Example output:
```
Attaching to 8 CPUs...
Sampling for 10 seconds...

PID		Samples
1234		450
5678		320
9101		156
...
```

## Use Cases

- Identifying CPU-intensive processes
- Lightweight system profiling
- Understanding CPU time distribution across processes
- Debugging performance issues with minimal overhead

## Notes

- Requires root/CAP_BPF/CAP_PERFMON capabilities
- The 99Hz sampling rate provides good profiling accuracy with low overhead
- The BPF map is limited to 10,240 unique PIDs
