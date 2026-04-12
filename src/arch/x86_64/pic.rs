use spin::Mutex;
use x86_64::instructions::interrupts;
use x86_64::instructions::port::Port;

/// PIC is Programmable Interrupt Controller, receives IRQ and tells it to CPU.
/// PIC itself is a dedicated hardware, controlled by this file via I/O port.

// PIC1 is the master PIC and handles the first 8 IRQ lines.
// PIC2 is the slave PIC and handles the next 8 IRQ lines.
pub const PIC1_OFFSET: u8 = 32;
pub const PIC2_OFFSET: u8 = PIC1_OFFSET + 8;

// Each PIC exposes a command port and a data port.
// These port numbers are the standard legacy x86 I/O port addresses for the 8259 PICs.
const PIC1_COMMAND: u16 = 0x0020;
const PIC1_DATA: u16 = 0x0021;
const PIC2_COMMAND: u16 = 0x00A0;
const PIC2_DATA: u16 = 0x00A1;

// EOI stands for End Of Interrupt.
// This command tells the PIC that interrupt handling has finished.
const PIC_EOI: u8 = 0x20;

// ICW1 is Initialization Command Word 1.
// This value tells the PIC to begin its initialization sequence.
const ICW1_INIT: u8 = 0x11;

// ICW4 configures additional PIC behavior.
// This value selects 8086/88 mode, which is the mode expected on modern x86 systems.
const ICW4_8086: u8 = 0x01;

// Keep interrupt vector indices as `u8` so `PIC1_OFFSET` can be used directly.
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

// This struct represents a single 8259 PIC chip.
// including its interrupt offset and its command/data I/O ports.
struct Pic {
    offset: u8,
    command: Port<u8>,
    data: Port<u8>,
}

impl Pic {
    const fn new(offset: u8, command_port: u16, data_port: u16) -> Self {
        Self {
            offset,
            command: Port::new(command_port),
            data: Port::new(data_port),
        }
    }

    // Returns true if this PIC is responsible for the given interrupt vector.
    fn handles_interrupt(&self, interrupt_id: u8) -> bool {
        self.offset <= interrupt_id && interrupt_id < self.offset + 8
    }
}

/// A pair of chained PICs.
/// The master PIC is connected directly to the CPU, and the slave PIC is connected through the master.
///
/// Classical x86_64 compatible PC is equipped with two PICs, so we prepare `Chained` struct.
pub struct ChainedPics {
    master: Pic,
    slave: Pic,
}

impl ChainedPics {
   pub const fn new(pic1_offset: u8, pic2_offset: u8) -> Self {
       Self {
           master: Pic::new(pic1_offset, PIC1_COMMAND, PIC1_DATA),
           slave: Pic::new(pic2_offset, PIC2_COMMAND, PIC2_DATA),
       }
   }

    // Remaps and initialize both PICs so their IRQs do not overwrap with CPU execution vectors.
    pub fn initialize(&mut self) {
        // Port 0x80 is traditionally used for a tiny I/O wait between PIC commands.
        let mut wait_port: Port<u8> = Port::new(0x80);

        let wait = |wait_port: &mut Port<u8>| unsafe {
            wait_port.write(0);
        };

        let saved_mask1: u8 = unsafe { self.master.data.read() };
        let saved_mask2: u8 = unsafe { self.slave.data.read() };

        unsafe { self.master.command.write(ICW1_INIT) };
        wait(&mut wait_port);
        unsafe { self.slave.command.write(ICW1_INIT) };
        wait(&mut wait_port);

        unsafe { self.master.data.write(self.master.offset) };
        wait(&mut wait_port);
        unsafe { self.slave.data.write(self.slave.offset) };
        wait(&mut wait_port);

        // Tell the master that the slave is connected ON IRQ2.
        unsafe { self.master.data.write(4) };
        wait(&mut wait_port);

        // Tell the slave its cascade identity.
        unsafe { self.slave.data.write(2) };
        wait(&mut wait_port);

        unsafe { self.master.data.write(ICW4_8086) };
        wait(&mut wait_port);
        unsafe { self.slave.data.write(ICW4_8086) };
        wait(&mut wait_port);

        unsafe { self.master.data.write(saved_mask1) };
        unsafe { self.slave.data.write(saved_mask2) };
    }

    // Sends End of Interrupt command to the PICs that handled the given interrupt.
    pub fn notify_end_of_interrupt(&mut self, interrupt_id: u8) {
        if self.slave.handles_interrupt(interrupt_id) {
            unsafe { self.slave.command.write(PIC_EOI) };
            unsafe { self.master.command.write(PIC_EOI) };
        } else if self.master.handles_interrupt(interrupt_id) {
            unsafe { self.master.command.write(PIC_EOI) };
        }
    }
}

// The PIC forwards external IRQs, Interrupt Requests, such as timer and keyboard interrupts to the CPU.
pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC1_OFFSET, PIC2_OFFSET) });

pub fn init_pics() {
    // Initialize the PIC before enabling external interrupts.
    interrupts::without_interrupts(|| unsafe {
        PICS.lock().initialize();
    });
}
