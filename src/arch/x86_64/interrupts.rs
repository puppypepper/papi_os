use crate::arch::x86_64::gdt::DOUBLE_FAULT_IST_INDEX;
use crate::arch::x86_64::hlt_loop;
use crate::arch::x86_64::pic::{InterruptIndex, PICS};
use crate::serial_println;
use core::sync::atomic::{AtomicU64, Ordering};
use lazy_static::lazy_static;
use x86_64::registers::control::Cr2;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

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

        idt[InterruptIndex::Timer.to_u8()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.to_u8()].set_handler_fn(keyboard_interrupt_handler);

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

static TIMER_TICKS: AtomicU64 = AtomicU64::new(0);
const TIMER_LOG_RATE: u64 = 100;

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    let tick = TIMER_TICKS.fetch_add(1, Ordering::Relaxed) + 1;

    // Print occasionally so the serial logs stays readable.
    if tick.is_multiple_of(TIMER_LOG_RATE) {
        serial_println!("timer tick: {}", tick);
    }

    // Notify the PIC that interrupt handling is complete.
    PICS.lock()
        .notify_end_of_interrupt(InterruptIndex::Timer.to_u8());
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    serial_println!("KEYBOARD");

    let mut pics = PICS.lock();

    // Reading port 0x60 drains the pending keyboard controller byte for this IRQ.
    // At this stage we only log the raw scancode and do not decode it yet.
    let scan_code: u8 = pics.read_scan_code();
    serial_println!("key_input: {:#04x}", scan_code);

    // Notify the PIC that interrupt handling is complete.
    pics.notify_end_of_interrupt(InterruptIndex::Keyboard.to_u8());
}
