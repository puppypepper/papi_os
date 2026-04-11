use crate::arch::x86_64::gdt::DOUBLE_FAULT_IST_INDEX;
use crate::arch::x86_64::hlt_loop;
use crate::arch::x86_64::pic::{PIC1_OFFSET, PICS};
use crate::serial_println;
use lazy_static::lazy_static;
use x86_64::registers::control::Cr2;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

// Keep interrupt vector indices as `u8` so `PIC1_OFFSET` can be used directly.
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    fn to_u8(self) -> u8 {
        self as u8
    }

    fn to_usize(self) -> usize {
        usize::from(self.to_u8())
    }
}

lazy_static! {
    pub static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        // The IDT stores the handler function addresses,
        // and the CPU jumps to the appropriate handler when an exception occurs.
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);

        // A double fault occurs when the CPU cannot invoke the handler for a prior exception.
        let double_fault_entry_options = idt.double_fault.set_handler_fn(double_fault_handler);
        unsafe {
            double_fault_entry_options.set_stack_index(DOUBLE_FAULT_IST_INDEX);
        }

        idt[InterruptIndex::Timer.to_usize()].set_handler_fn(timer_interrupt_handler);

        idt
    };
}

pub fn init_idt() {
    // Load the IDT pointer into the IDTR register.
    IDT.load();
    serial_println!("IDT loaded");
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    serial_println!("EXCEPTION: BREAKPOINT");
    serial_println!("{:#?}", stack_frame);
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    serial_println!("EXCEPTION: PAGE FAULT");
    // Cr2 register is an x86_64's special register which contains the most recent page fault address.
    serial_println!("Accessed Address: {:?}", Cr2::read());
    serial_println!("Error Code: {:?}", error_code);
    serial_println!("{:#?}", stack_frame);

    hlt_loop();
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    serial_println!("EXCEPTION: DOUBLE FAULT");
    serial_println!("{:#?}", stack_frame);

    hlt_loop();
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    // Keep the timer handler minimum until the interrupt path is verified.
    // serial_println!("TIMER");

    // Notify the PIC that interrupt handling is complete.
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.to_u8());
    }
}
