#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(lithos::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use lithos::println;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    lithos::serial_println!("Breakpoint test _start");
    lithos::init();
    test_main();

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    lithos::test_panic_handler(info)
}

#[test_case]
fn test_breakpoint_exception() {
    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
}
