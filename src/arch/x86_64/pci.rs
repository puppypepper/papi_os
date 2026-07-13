use x86_64::instructions::port::Port;
use crate::serial_println;

const MAX_BUS: u8 = 255;
const MAX_DEVICE: u8 = 31;
const MAX_FUNC: u8 = 7;

const CONFIG_ADDRESS: u16 = 0x0CF8;
const CONFIG_DATA: u16 = 0x0CFC;

struct PciDeviceConfig {
    vendor_id: u16,
    device_id: u16,
    class: u8,
    subclass: u8,
    header_type: u8,
}

pub fn scan_pci_bus() {
    for bus in 0..=MAX_BUS {
        for device in 0..=MAX_DEVICE {
            for func in 0..=MAX_FUNC {
                let data_0: u32 = read_config(bus, device, func, 0x00);

                let vendor_id: u16 = (data_0 & 0x0000_FFFF) as u16;
                if vendor_id == 0xFFFF {
                    continue;
                }
                let device_id: u16 = (data_0 >> 16) as u16;

                let data_8: u32 = read_config(bus, device, func, 0x08);
                let class: u8 = (data_8 >> 24) as u8;
                let subclass: u8 = (data_8 >> 16 & 0x00FF) as u8;

                let data_c: u32 = read_config(bus, device, func, 0x0c);
                let header_type: u8 = (data_c >> 16 & 0x00FF) as u8;

                let _config = PciDeviceConfig {
                    vendor_id,
                    device_id,
                    class,
                    subclass,
                    header_type,
                };

                serial_println!(
    "[PCI] {:02x}:{:02x}.{} vendor={:#06x} device={:#06x} class={:#04x} sub={:#04x}",
    bus, device, func, vendor_id, device_id, class, subclass
);
            }
        }
    }
}

fn read_config(bus: u8, device: u8, func: u8, offset: u8) -> u32 {
    let mut address_port: Port<u32> = Port::new(CONFIG_ADDRESS);
    let mut data_port: Port<u32> = Port::new(CONFIG_DATA);

    // build Port IO data
    let address: u32 = 0x8000_0000
        | ((bus as u32) << 16)
        | ((device as u32) << 11)
        | ((func as u32) << 8)
        | ((offset as u32) & 0x0000_00FC);

    // write to address port
    unsafe {
        address_port.write(address);
    };

    // read from data port
    let port_data: u32;
    unsafe {
        port_data = data_port.read();
    }

    port_data
}