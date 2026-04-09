use pic8259_simple::ChainedPics;
use spin::Mutex;

/// PIC is Programmable Interrupt Controller

pub const PIC1_OFFSET: u8 = 32;
pub const PIC2_OFFSET: u8 = PIC1_OFFSET + 8;

// The PIC forwards external IRQs such as timer and keyboard interrupts to the CPU.
pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC1_OFFSET, PIC2_OFFSET) });

pub fn init_pics() {
    // Remap and initialize the PIC before enabling external interrupts.
    unsafe {
        PICS.lock().initialize();
    }
}