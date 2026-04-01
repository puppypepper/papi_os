use core::arch::asm;

#[inline]
fn hlt() {
    unsafe { asm!("hlt"); }
}

pub fn hlt_loop() -> ! {
    loop {
        hlt();
    }
}