cmake_minimum_required(VERSION 3.16)
project(exec_tracer C)

set(CMAKE_C_STANDARD 11)

# Find required programs
find_program(CLANG clang REQUIRED)
find_program(BPFTOOL bpftool REQUIRED)

# Find libbpf
find_library(LIBBPF_LIBRARY bpf REQUIRED)

# BPF source and output files
set(BPF_SOURCE ${CMAKE_CURRENT_SOURCE_DIR}/exec.bpf.c)
set(BPF_OBJECT ${CMAKE_CURRENT_BINARY_DIR}/exec.bpf.o)
set(BPF_SKEL ${CMAKE_CURRENT_BINARY_DIR}/exec.skel.h)

# Detect target architecture
execute_process(
    COMMAND uname -m
    OUTPUT_VARIABLE ARCH
    OUTPUT_STRIP_TRAILING_WHITESPACE
)

if(ARCH STREQUAL "x86_64")
    set(TARGET_ARCH "x86")
elseif(ARCH STREQUAL "aarch64")
    set(TARGET_ARCH "arm64")
else()
    set(TARGET_ARCH ${ARCH})
endif()

# Custom command to compile BPF program
add_custom_command(
    OUTPUT ${BPF_OBJECT}
    COMMAND ${CLANG} -O2 -g -target bpf -D__TARGET_ARCH_${TARGET_ARCH}
            -c ${BPF_SOURCE} -o ${BPF_OBJECT}
    DEPENDS ${BPF_SOURCE}
    COMMENT "Compiling BPF program: ${BPF_SOURCE}"
    VERBATIM
)

# Custom command to generate BPF skeleton
add_custom_command(
    OUTPUT ${BPF_SKEL}
    COMMAND ${BPFTOOL} gen skeleton ${BPF_OBJECT} > ${BPF_SKEL}
    DEPENDS ${BPF_OBJECT}
    COMMENT "Generating BPF skeleton: ${BPF_SKEL}"
    VERBATIM
)

# Add custom target for BPF artifacts
add_custom_target(bpf_artifacts ALL
    DEPENDS ${BPF_OBJECT} ${BPF_SKEL}
)

# Main executable
add_executable(exec exec.c)

# Add dependency on BPF artifacts
add_dependencies(exec bpf_artifacts)

# Include directory for generated skeleton
target_include_directories(exec PRIVATE ${CMAKE_CURRENT_BINARY_DIR})

# Link against libbpf
target_link_libraries(exec ${LIBBPF_LIBRARY})

# Install target
install(TARGETS exec DESTINATION bin)
