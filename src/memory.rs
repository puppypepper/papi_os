pub(crate) mod page_mapper;
mod frame_allocator;

use crate::serial_println;
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::registers::control::{Cr3, Cr3Flags};
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageSize, PageTable, PageTableFlags, PhysFrame,
    Size4KiB,
};
use x86_64::{PhysAddr, VirtAddr};
pub(crate) use crate::memory::frame_allocator::BootInfoFrameAllocator;
pub(crate) use crate::memory::page_mapper::{PageMapper, PhysicalMemoryOffest};

// Read CR3 to find the currently active level 4 page table frame, then convert
// that physical address into a virtual address through the bootloader's
// "physical memory is mapped at this offset" contract.
//
// Why "level 4"?
// In x86_64 long mode, normal 4 KiB paging uses a 4-level hierarchy:
//
//   level 4 -> level 3 -> level 2 -> level 1 -> final 4 KiB page
//
// CR3 always points to the top-level table of the current address space, so
// the first table we recover from the CPU is the level 4 table.
unsafe fn active_level4_table(
    physical_memory_offest: &PhysicalMemoryOffest,
) -> &'static mut PageTable {
    let (level4_table_frame, _): (PhysFrame, Cr3Flags) = Cr3::read();
    let physical_address: PhysAddr = level4_table_frame.start_address();
    let virtual_address: VirtAddr =
        physical_memory_offest.as_virt_addr() + physical_address.as_u64();
    let page_table_ptr: *mut PageTable = virtual_address.as_mut_ptr();

    &mut *page_table_ptr
}

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

// Expand `Usable` memory regions into an iterator of 4 KiB physical frames.
fn usable_frame(memory_map: &'static MemoryMap) -> impl Iterator<Item = PhysFrame> {
    memory_map
        .iter()
        .filter(|region| region.region_type == MemoryRegionType::Usable)
        .flat_map(|region| region.range.start_frame_number..region.range.end_frame_number)
        .map(|frame_number| frame_number * Size4KiB::SIZE)
        .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
}
