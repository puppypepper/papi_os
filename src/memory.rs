mod frame_allocator;
mod heap_allocator;
pub(crate) mod page_mapper;

pub(crate) use crate::memory::frame_allocator::BootInfoFrameAllocator;
pub(crate) use crate::memory::heap_allocator::init_heap;
pub(crate) use crate::memory::page_mapper::{PageMapper, PhysicalMemoryOffest};
use crate::serial_println;
use bootloader::bootinfo::MemoryMap;

// Log the physical memory regions that the bootloader reported to the kernel.
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
