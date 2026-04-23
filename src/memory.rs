use crate::serial_println;
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::PhysAddr;
use x86_64::structures::paging::{FrameAllocator, PageSize, PhysFrame, Size4KiB};

pub fn print_memory_map(memory_map: &MemoryMap) {
    serial_println!("=== START memory map information === ");
    for region in memory_map.iter() {
        let start = region.range.start_addr();
        let end = region.range.end_addr();
        let kind = region.region_type;

        serial_println!("{:#018x}..{:#018x} {:#?}", start, end, kind);
    }
    serial_println!("=== END memory map information === ");
}

fn usable_frame(memory_map: &'static MemoryMap) -> impl Iterator<Item = PhysFrame> {
    memory_map
        .iter()
        .filter(|region| region.region_type == MemoryRegionType::Usable)
        .flat_map(|region| region.range.start_frame_number..region.range.end_frame_number)
        .map(|frame_number| frame_number * Size4KiB::SIZE)
        .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
}

pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    pub fn init(memory_map: &'static MemoryMap) -> Self {
        Self {
            memory_map,
            next: 0,
        }
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let frame = usable_frame(self.memory_map).nth(self.next);
        self.next += 1;
        frame
    }
}