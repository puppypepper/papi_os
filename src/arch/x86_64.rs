pub mod gdt;
pub mod interrupts;
pub mod pci;
pub mod pic;

use core::arch::asm;

#[inline]
fn hlt() {
    unsafe {
        asm!("hlt");
    }
}

pub fn hlt_loop() -> ! {
    loop {
        hlt();
    }
}
