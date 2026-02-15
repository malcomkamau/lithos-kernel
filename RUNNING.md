# Running Lithos OS

## Prerequisites

Make sure you have the following installed:
```bash
# Install QEMU
sudo apt install qemu-system-x86_64  # Ubuntu/Debian
# or
brew install qemu  # macOS

# Install bootimage tool (if not already installed)
cargo install bootimage
```

## Building the OS

```bash
# Build the kernel and create bootable image
cargo bootimage --release
```

This creates a bootable disk image at:
`target/x86_64-lithos/release/bootimage-lithos.bin`

## Running with QEMU

### Basic Launch
```bash
qemu-system-x86_64 -drive format=raw,file=target/x86_64-lithos/release/bootimage-lithos.bin
```

### Recommended Launch (with more features)
```bash
qemu-system-x86_64 \
  -drive format=raw,file=target/x86_64-lithos/release/bootimage-lithos.bin \
  -serial mon:stdio \
  -m 256M \
  -cpu max \
  -smp 2
```

**Options explained:**
- `-drive format=raw,file=...` - Boot from our disk image
- `-serial mon:stdio` - Redirect serial output to terminal (for `println!`)
- `-m 256M` - Allocate 256MB RAM
- `-cpu max` - Use maximum CPU features
- `-smp 2` - 2 CPU cores

### With Disk Support (for ATA driver testing)
```bash
# Create a test disk image (100MB)
qemu-img create -f raw disk.img 100M

# Run with additional disk
qemu-system-x86_64 \
  -drive format=raw,file=target/x86_64-lithos/release/bootimage-lithos.bin \
  -drive format=raw,file=disk.img \
  -serial mon:stdio \
  -m 256M
```

### Debug Mode (with GDB)
```bash
qemu-system-x86_64 \
  -drive format=raw,file=target/x86_64-lithos/release/bootimage-lithos.bin \
  -serial mon:stdio \
  -s -S
```

Then in another terminal:
```bash
gdb target/x86_64-lithos/release/lithos
(gdb) target remote :1234
(gdb) continue
```

## Quick Start Script

Create a `run.sh` file:
```bash
#!/bin/bash
cargo bootimage --release && \
qemu-system-x86_64 \
  -drive format=raw,file=target/x86_64-lithos/release/bootimage-lithos.bin \
  -serial mon:stdio \
  -m 256M \
  -cpu max
```

Make it executable and run:
```bash
chmod +x run.sh
./run.sh
```

## Expected Output

You should see:
```
=== Lithos OS Boot ===

Testing Block Device Layer...
  ✓ Wrote block 0
  ✓ Read block 0 - data matches!
  RAM Disk: 100 blocks (50 KB)

Initializing Virtual File System...
VFS initialized with ramfs

=== Testing VFS Operations ===
[... file system operations ...]

=== Lithos Shell Demo ===
[... shell commands ...]

Lithos OS is fully operational!
Features: Multitasking, VFS, Block Devices, FAT32, Device Files, Shell
```

## Exiting QEMU

- Press `Ctrl+A` then `X` to exit QEMU
- Or close the QEMU window

## Troubleshooting

**Issue**: `cargo bootimage` not found
```bash
cargo install bootimage
rustup component add llvm-tools-preview
```

**Issue**: QEMU not found
```bash
# Ubuntu/Debian
sudo apt install qemu-system-x86_64

# macOS
brew install qemu

# Arch Linux
sudo pacman -S qemu
```

**Issue**: Build fails
```bash
# Make sure you're using nightly Rust
rustup override set nightly
cargo clean
cargo bootimage --release
```
