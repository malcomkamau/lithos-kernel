#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(lithos::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;
use lithos::{println, print};
use bootloader::{BootInfo, entry_point};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use lithos::memory;
    use x86_64::{VirtAddr, structures::paging::Translate};

    println!("Lithos Kernel OS booting...");
    lithos::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe {
        memory::BootInfoFrameAllocator::init(&boot_info.memory_map)
    };

    lithos::allocator::init_heap(&mut mapper, &mut frame_allocator)
        .expect("heap initialization failed");

    use alloc::{boxed::Box, vec, vec::Vec, rc::Rc};

    let heap_value = Box::new(41);
    println!("heap_value at {:p}", heap_value);

    let mut vec = Vec::new();
    for i in 0..500 {
        vec.push(i);
    }
    println!("vec at {:p}", vec.as_slice());

    let reference_counted = Rc::new(vec![1, 2, 3]);
    let cloned_reference = reference_counted.clone();
    println!("current reference count is {}", Rc::strong_count(&cloned_reference));
    core::mem::drop(reference_counted);
    println!("reference count is {} now", Rc::strong_count(&cloned_reference));

    // translate some addresses
    let addresses = [
        // the identity-mapped vga buffer page
        0xb8000,
        // some code page
        0x201008,
        // some stack page
        0x0100_0020_1a10,
        // virtual address mapped to physical address 0
        boot_info.physical_memory_offset,
    ];

    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys = mapper.translate_addr(virt);
        println!("{:?} -> {:?}", virt, phys);
    }

    println!("It did not crash!");

    #[cfg(test)]
    test_main();

    println!("=== Lithos OS Boot ===\n");
    
    // Test block device layer
    println!("Testing Block Device Layer...");
    use lithos::drivers::block::{ramdisk::RamDisk, BlockDevice};
    
    let mut ramdisk = RamDisk::new(100); // 100 blocks = 51.2 KB
    let mut write_buf = [0u8; 512];
    let mut read_buf = [0u8; 512];
    
    // Write test data
    for i in 0..512 {
        write_buf[i] = (i % 256) as u8;
    }
    
    match ramdisk.write_block(0, &write_buf) {
        Ok(_) => println!("  ✓ Wrote block 0"),
        Err(e) => println!("  ✗ Write failed: {}", e),
    }
    
    // Read it back
    match ramdisk.read_block(0, &mut read_buf) {
        Ok(_) => {
            if read_buf == write_buf {
                println!("  ✓ Read block 0 - data matches!");
            } else {
                println!("  ✗ Read block 0 - data mismatch!");
            }
        }
        Err(e) => println!("  ✗ Read failed: {}", e),
    }
    
    println!("  RAM Disk: {} blocks ({} KB)\n", ramdisk.block_count(), ramdisk.block_count() / 2);

    println!("Initializing Virtual File System...");
    
    use lithos::vfs::{ramfs::RamFs, ops};
    
    // Create and initialize ramfs
    let ramfs = RamFs::new();
    let root = ramfs.root_node();
    ops::init(root);
    
    println!("VFS initialized with ramfs");
    
    // Test VFS operations
    println!("\n=== Testing VFS Operations ===");
    
    // Create directories
    println!("Creating directories...");
    match ops::vfs_mkdir("/home") {
        Ok(_) => println!("  ✓ Created /home"),
        Err(e) => println!("  ✗ Failed to create /home: {}", e),
    }
    
    match ops::vfs_mkdir("/home/user") {
        Ok(_) => println!("  ✓ Created /home/user"),
        Err(e) => println!("  ✗ Failed to create /home/user: {}", e),
    }
    
    match ops::vfs_mkdir("/tmp") {
        Ok(_) => println!("  ✓ Created /tmp"),
        Err(e) => println!("  ✗ Failed to create /tmp: {}", e),
    }
    
    // Create files
    println!("\nCreating files...");
    match ops::vfs_create("/home/user/test.txt") {
        Ok(_) => println!("  ✓ Created /home/user/test.txt"),
        Err(e) => println!("  ✗ Failed to create file: {}", e),
    }
    
    match ops::vfs_create("/home/user/readme.md") {
        Ok(_) => println!("  ✓ Created /home/user/readme.md"),
        Err(e) => println!("  ✗ Failed to create file: {}", e),
    }
    
    // List directory contents
    println!("\nListing directory contents...");
    match ops::vfs_readdir("/") {
        Ok(entries) => {
            println!("  Contents of /:");
            for entry in entries {
                println!("    - {}", entry);
            }
        }
        Err(e) => println!("  ✗ Failed to read /: {}", e),
    }
    
    match ops::vfs_readdir("/home") {
        Ok(entries) => {
            println!("  Contents of /home:");
            for entry in entries {
                println!("    - {}", entry);
            }
        }
        Err(e) => println!("  ✗ Failed to read /home: {}", e),
    }
    
    match ops::vfs_readdir("/home/user") {
        Ok(entries) => {
            println!("  Contents of /home/user:");
            for entry in entries {
                println!("    - {}", entry);
            }
        }
        Err(e) => println!("  ✗ Failed to read /home/user: {}", e),
    }
    
    println!("\n=== VFS Test Complete ===");
    
    // Create /dev directory and add device files
    println!("\n=== Setting up Device Files ===");
    match ops::vfs_mkdir("/dev") {
        Ok(_) => println!("  ✓ Created /dev"),
        Err(e) => println!("  ✗ Failed to create /dev: {}", e),
    }
    
    // Test device files
    println!("\nTesting device files...");
    use lithos::vfs::devfs;
    
    let dev_nodes = devfs::create_dev_nodes();
    println!("  Created {} device nodes", dev_nodes.len());
    for (name, _node) in &dev_nodes {
        println!("    - /dev/{}", name);
    }
    
    // Test /dev/zero
    let zero_node = &dev_nodes[1].1;
    let mut buf = [0xFFu8; 16];
    match zero_node.lock().read_at(0, &mut buf) {
        Ok(n) => {
            print!("  /dev/zero read {} bytes: ", n);
            for b in &buf[..8] {
                print!("{:02x} ", b);
            }
            println!("...");
        }
        Err(e) => println!("  /dev/zero read failed: {}", e),
    }
    
    // Test /dev/random
    let random_node = &dev_nodes[2].1;
    match random_node.lock().read_at(0, &mut buf) {
        Ok(n) => {
            print!("  /dev/random read {} bytes: ", n);
            for b in &buf[..8] {
                print!("{:02x} ", b);
            }
            println!("...");
        }
        Err(e) => println!("  /dev/random read failed: {}", e),
    }
    
    // Create initial directory structure
    println!("\n=== Creating Initial Directory Structure ===");
    let _ = ops::vfs_mkdir("/usr");
    let _ = ops::vfs_mkdir("/usr/bin");
    let _ = ops::vfs_mkdir("/etc");
    let _ = ops::vfs_mkdir("/var");
    println!("  ✓ Created /usr, /usr/bin, /etc, /var");
    
    // Interactive shell
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║            Welcome to Lithos OS v0.1.0                   ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();
    println!("Features: Multitasking, VFS, Block Devices, FAT32, Syscalls");
    println!("Type 'help' for available commands");
    println!();
    
    use lithos::shell::Shell;
    let mut shell = Shell::new();
    
    // Interactive shell loop
    loop {
        print!("lithos$ ");
        
        // In a real implementation, we would read from keyboard
        // For now, we'll demonstrate with predefined commands
        // This is where keyboard input would be integrated
        
        // Since we don't have keyboard input yet, run a demo sequence
        println!("(Interactive mode not yet implemented - running demo)");
        println!();
        
        let demo_commands = [
            "help",
            "pwd",
            "ls /",
            "cd /usr",
            "pwd",
            "ls /usr",
            "mkdir /usr/local",
            "touch /usr/local/test.txt",
            "ls /usr/local",
            "cd /",
            "echo Lithos OS is running!",
        ];
        
        for cmd in &demo_commands {
            println!("lithos$ {}", cmd);
            shell.execute(cmd);
            println!();
        }
        
        println!("\n╔═══════════════════════════════════════════════════════════╗");
        println!("║  Interactive keyboard input coming soon!                 ║");
        println!("║  For now, the OS runs in demo mode.                      ║");
        println!("╚═══════════════════════════════════════════════════════════╝");
        
        break; // Exit after demo
    }
    
    println!("\nLithos OS demonstration complete!");
    println!("Press Ctrl+A then X to exit QEMU");
    
    lithos::hlt_loop();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    lithos::test_panic_handler(info)
}
