use spin::Mutex;
use x86_64::instructions::interrupts;
use x86_64::instructions::port::Port;

/// PIC is Programmable Interrupt Controller

pub const PIC1_OFFSET: u8 = 32;
pub const PIC2_OFFSET: u8 = PIC1_OFFSET + 8;

const PIC1_COMMAND: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_COMMAND: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;

const PIC_EOI: u8 = 0x20;
const ICW1_INIT: u8 = 0x11;
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

    fn handles_interrupt(&self, interrupt_id: u8) -> bool {
        // TODO comment
        self.offset <= interrupt_id && interrupt_id < self.offset + 8
    }
}

/// TODO comment
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

    // TODO comment
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
