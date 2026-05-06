#![feature(abi_x86_interrupt)]
#![no_std] // don't link the Rust standard library
#![no_main] // disable all Rust-level entry points

use crate::arch::x86_64::gdt::init_gdt;
use crate::arch::x86_64::hlt_loop;
use crate::arch::x86_64::interrupts::init_idt;
use crate::arch::x86_64::pic::init_pics;
use crate::memory::{BootInfoFrameAllocator, PageMapper, PhysicalMemoryOffest};
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use x86_64::structures::paging::FrameAllocator;

// `entry_point!` generates the `_start` symbol with the ABI expected by the
// bootloader and passes startup information as `&'static BootInfo`.
entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    init_gdt();
    init_idt();
    init_pics();

    // Inspect the physical memory layout that the bootloader handed to us.
    memory::print_memory_map(&boot_info.memory_map);

    // Build a simple allocator that hands out 4 KiB physical frames from
    // regions marked `Usable` in the bootloader's memory map.
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    // The bootloader mapped physical memory into the kernel's virtual address
    // space at a fixed offset and reports that offset through `BootInfo`.
    // From this point on, paging code can translate:
    //
    //     virtual address = physical address + physical_memory_offset
    // That is what lets the kernel access page tables and other physical
    // memory-backed structures through normal virtual addresses.
    let physical_memory_offset = PhysicalMemoryOffest::new(boot_info.physical_memory_offset);
    serial_println!(
        "physical memory offset: {:#018x}",
        &physical_memory_offset.as_u64()
    );

    // `PageMapper` is the facade for "the currently active paging state".
    // Creating it means: find the page tables that are already active on the
    // CPU and wrap them in an object that later code can use to add or inspect
    // mappings.
    let mut page_mapper: PageMapper = unsafe { memory::init(&physical_memory_offset) };

    // Enable hardware interrupts only after the GDT, IDT, and PIC are ready.
    x86_64::instructions::interrupts::enable();

    // Allocate a few frames as a smoke test and log their physical addresses.
    for _ in 0..5 {
        if let Some(frame) = frame_allocator.allocate_frame() {
            serial_println!("allocated frame: {:#018x}", frame.start_address().as_u64());
        }
    }

    let demo_virt_addr_raw: u64 = 0x4444_4444_0000;
    page_mapper.map_page(demo_virt_addr_raw, &mut frame_allocator);
    let vptr = demo_virt_addr_raw as *mut u64;
    unsafe {
        let val: u64 = 0x_f021_f077_f065_f04e;
        *vptr = val;
        serial_println!("val:  {:#018x}", val);
        serial_println!("vptr: {:#018x}", *vptr);
    }

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
mod memory;
mod serial;
mod vga_buffer;
