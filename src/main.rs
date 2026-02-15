#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(lithos::test_runner)]
#![reexport_test_harness_main = "test_main"]

extern crate alloc;

use core::panic::PanicInfo;
use lithos::println;
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

    use lithos::task::{Task, executor::Executor, keyboard};

    let mut executor = Executor::new();
    
    // Spawn keyboard task
    executor.spawn(Task::new(keyboard::print_keypresses()));
    
    // Spawn a CPU-bound task to demonstrate preemptive scheduling
    executor.spawn(Task::new(async {
        let mut counter = 0u64;
        loop {
            counter = counter.wrapping_add(1);
            if counter % 100_000 == 0 {
                lithos::println!("Task A: {}", counter);
            }
            // Yield to allow other tasks to run
            core::future::poll_fn(|cx| {
                cx.waker().wake_by_ref();
                core::task::Poll::<()>::Pending
            }).await;
        }
    }));
    
    // Spawn another CPU-bound task
    executor.spawn(Task::new(async {
        let mut counter = 0u64;
        loop {
            counter = counter.wrapping_add(1);
            if counter % 150_000 == 0 {
                lithos::println!("Task B: {}", counter);
            }
            // Yield to allow other tasks to run
            core::future::poll_fn(|cx| {
                cx.waker().wake_by_ref();
                core::task::Poll::<()>::Pending
            }).await;
        }
    }));
    
    // Initialize global executor
    lithos::task::executor::init(executor);
    
    // Run the executor
    if let Some(executor) = lithos::task::executor::get_executor() {
        executor.lock().run();
    } else {
        panic!("Failed to initialize executor");
    }
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
