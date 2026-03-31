use core::fmt;
use core::fmt::Write;
use lazy_static::lazy_static;
use spin::Mutex;
use uart_16550::SerialPort;

lazy_static! {
    pub static ref SERIAL1: Mutex<SerialPort> = {
        let mut serial_port = unsafe { SerialPort::new(0x3F8) };
        serial_port.init();
        Mutex::new(serial_port)
    };
}

pub fn _print(args: fmt::Arguments) {
    SERIAL1.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::serial::_print(format_args!($($arg)*));
    };
}

#[macro_export]
macro_rules! serial_println {
    () => {
        $crate::serial_print!("\r\n");
    };
    ($fmt:expr) => {
        $crate::serial_print!(concat!($fmt, "\r\n"));
    };
    ($fmt:expr, $($arg:tt)*) => {
        $crate::serial_print!(concat!($fmt, "\r\n"), $($arg)*);
    }
}