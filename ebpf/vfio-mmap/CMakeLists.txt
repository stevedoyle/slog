cmake_minimum_required(VERSION 3.16)
project(vfio-mmap-tracer C)

set(CMAKE_C_STANDARD 11)

# Define the first eBPF program (VFIO IOCTL tracing)
set(BPF_SOURCE1 iommu_mmap_trace.bpf.c)
set(BPF_OBJECT1 iommu_mmap_trace.bpf.o)

# Custom command to compile the first eBPF program
add_custom_command(
    OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/${BPF_OBJECT1}
    COMMAND clang -O2 -g -target bpf -D__TARGET_ARCH_x86
            -I${CMAKE_CURRENT_SOURCE_DIR} -c
            ${CMAKE_CURRENT_SOURCE_DIR}/${BPF_SOURCE1}
            -o ${CMAKE_CURRENT_BINARY_DIR}/${BPF_OBJECT1}
    DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/${BPF_SOURCE1}
    COMMENT "Compiling eBPF program: ${BPF_SOURCE1}"
    VERBATIM
)

# Custom target for the first eBPF object file
add_custom_target(bpf_object_mmap ALL
    DEPENDS ${CMAKE_CURRENT_BINARY_DIR}/${BPF_OBJECT1}
)

# Define the second eBPF program (IOMMU domain tracing)
set(BPF_SOURCE2 iommu_domain_mmap.bpf.c)
set(BPF_OBJECT2 iommu_domain_mmap.bpf.o)

# Custom command to compile the second eBPF program
add_custom_command(
    OUTPUT ${CMAKE_CURRENT_BINARY_DIR}/${BPF_OBJECT2}
    COMMAND clang -O2 -g -target bpf -D__TARGET_ARCH_x86
            -I${CMAKE_CURRENT_SOURCE_DIR} -c
            ${CMAKE_CURRENT_SOURCE_DIR}/${BPF_SOURCE2}
            -o ${CMAKE_CURRENT_BINARY_DIR}/${BPF_OBJECT2}
    DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/${BPF_SOURCE2}
    COMMENT "Compiling eBPF program: ${BPF_SOURCE2}"
    VERBATIM
)

# Custom target for the second eBPF object file
add_custom_target(bpf_object_domain ALL
    DEPENDS ${CMAKE_CURRENT_BINARY_DIR}/${BPF_OBJECT2}
)

# Build the first loader executable (VFIO IOCTL)
add_executable(loader_mmap loader_mmap.c)
target_link_libraries(loader_mmap bpf)
add_dependencies(loader_mmap bpf_object_mmap)

# Build the second loader executable (IOMMU domain)
add_executable(loader_domain loader_domain.c)
target_link_libraries(loader_domain bpf)
add_dependencies(loader_domain bpf_object_domain)

# Install targets (optional)
install(TARGETS loader_mmap loader_domain DESTINATION bin)
install(FILES ${CMAKE_CURRENT_BINARY_DIR}/${BPF_OBJECT1}
              ${CMAKE_CURRENT_BINARY_DIR}/${BPF_OBJECT2}
        DESTINATION share/vfio-mmap-tracer)
