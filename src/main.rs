#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(lithos::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use lithos::println;
#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("Lithos Kernel OS booting...");

    lithos::init();

    println!("It did not crash!");

    #[cfg(test)]
    test_main();

    loop {}
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
