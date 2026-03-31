#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

use core::panic::PanicInfo;

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    vga_buffer::print_something();

    serial_println!("kernel booted");
    serial_println!("hello from serial");

    // this function is the entry point, since the linker looks for a function
    // named `_start` by default
    loop {}
}

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!();
    serial_println!("================ PANIC ================");
    serial_println!("{}", info);
    
    loop {}
}

mod vga_buffer;
mod serial;
