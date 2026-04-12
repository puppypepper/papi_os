#![feature(abi_x86_interrupt)]
#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

use crate::arch::x86_64::gdt::init_gdt;
use crate::arch::x86_64::hlt_loop;
use crate::arch::x86_64::interrupts::init_idt;
use crate::arch::x86_64::pic::init_pics;
use core::arch::asm;
use core::panic::PanicInfo;

// this function is the entry point, since the linker looks for a function
// named `_start` by default
#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    init_gdt();
    init_idt();
    init_pics();

    // Enable hardware interrupts only after the GDT, IDT, and PIC are ready.
    x86_64::instructions::interrupts::enable();

    vga_buffer::print_something();

    serial_println!("kernel booted");
    serial_println!("hello from serial");

    // `int3` instruction triggers breakpoint exception on x86_64.
    // unsafe {
    //     asm!("int3");
    // }

    hlt_loop()
}

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!();
    serial_println!("================ PANIC ================");
    serial_println!("{}", info);

    hlt_loop()
}

mod arch;
mod serial;
mod vga_buffer;
