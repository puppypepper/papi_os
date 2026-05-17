mod frame_allocator;
pub(crate) mod page_mapper;

pub(crate) use crate::memory::frame_allocator::BootInfoFrameAllocator;
pub(crate) use crate::memory::page_mapper::{PageMapper, PhysicalMemoryOffest};
use crate::serial_println;
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::registers::control::{Cr3, Cr3Flags};
use x86_64::structures::paging::{PageSize, PageTable, PhysFrame, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};

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
