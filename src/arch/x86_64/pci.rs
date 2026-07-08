use x86_64::instructions::port::Port;

const MAX_BUS: u8 = 255;
const MAX_DEVICE: u8 = 32;
const MAX_FUNC: u8 = 8;

const CONFIG_ADDRESS: u16 = 0x0CF8;
const CONFIG_DATA: u16 = 0x0CFC;

struct PciDeviceConfig {
    vendor_id: u16,
    device_id: u16,
    class: u8,
    subclass: u8,
    header_type: u8,
    rest: u32,
}

pub fn scan_pci_bus() {
    for bus in 0..=MAX_BUS {
        for device in 0..=MAX_DEVICE {
            for func in 0..=MAX_FUNC {
                let pci_device_config: PciDeviceConfig = read_config(bus, device, func, 0x00);

            }
        }
    }
}

fn read_config(bus: u8, device: u8, func: u8, offset: u8) -> PciDeviceConfig {
    let mut address_port: Port<u32> = Port::new(CONFIG_ADDRESS);
    let mut data_port: Port<u32> = Port::new(CONFIG_DATA);

    // build Port IO data

    // write to address port

    // read from data port

    todo!()
}