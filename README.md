# Lithos OS
Lithos is an ambitious x86_64 microkernel-architected operating system being built from the ground up in Rust.

## Vision
Lithos aims to provide a safe, modular, and high-performance foundation for modern computing, leveraging Rust's zero-cost abstractions and memory safety.

## Current Progress
- [x] **Bare-metal Environment**: Custom target specification and `no_std` environment.
- [x] **VGA Text Driver**: Safe interface for screen output.
- [x] **Serial Logging**: Professional-grade debugging interface.
- [x] **Testing Infrastructure**: Integrated framework for verified development.
- [x] **CPU Foundation**: GDT, IDT, and Interrupt Handling.
- [x] **Memory Management**: Paging, Frame Allocation, and Heap Support.
- [x] **Multitasking**: Context switching with CPU register preservation and timer-based preemption.
- [x] **Virtual File System**: Unified file interface with ramfs implementation.
- [x] **Block Device Layer**: Abstract disk I/O with RAM disk support.
- [x] **FAT32 Support**: Boot sector parsing and directory structures (read-only foundation).
- [x] **Device Files**: /dev/null, /dev/zero, /dev/random.
- [x] **Interactive Shell**: Command-line interface with ls, mkdir, cd, touch, echo, and more.
- [x] **System Calls**: Full syscall interface (read, write, open, close, exit, fork, exec, wait).
- [x] **ATA/IDE Driver**: Physical disk access via PIO mode.
- [x] **ELF Loader**: Parse and load ELF64 binaries.
