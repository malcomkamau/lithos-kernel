#!/bin/bash
# Direct QEMU launch script for Lithos OS
# This bypasses bootimage and uses the kernel binary directly

set -e

echo "🔨 Building Lithos OS..."
cargo build --release

echo "✅ Build complete!"
echo ""
echo "📝 To run Lithos OS in QEMU, you have two options:"
echo ""
echo "Option 1: Install bootimage properly"
echo "  cargo install bootimage"
echo "  rustup component add llvm-tools-preview"
echo "  cargo bootimage --release"
echo "  qemu-system-x86_64 -drive format=raw,file=target/x86_64-unknown-none/release/bootimage-lithos.bin"
echo ""
echo "Option 2: Use bootloader crate directly (recommended)"
echo "  The kernel is built at: target/x86_64-unknown-none/release/lithos"
echo "  You need a bootloader to create a bootable image"
echo ""
echo "💡 Quick fix: Try running with the bootloader's test runner:"
echo "  cargo test --no-fail-fast"
echo ""
