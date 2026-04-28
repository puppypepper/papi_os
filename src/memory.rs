use crate::serial_println;
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::structures::paging::{FrameAllocator, OffsetPageTable, PageSize, PageTable, PhysFrame, Size4KiB};
use x86_64::{PhysAddr, VirtAddr};
use x86_64::registers::control::{Cr3, Cr3Flags};

pub struct PhysicalMemoryOffest(u64);

impl PhysicalMemoryOffest {
    pub const fn new(offset: u64) -> Self {
        Self(offset)
    }

    fn as_virt_addr(&self) -> VirtAddr {
        VirtAddr::new(self.0)
    }
}

pub struct PageMapper {
    _inner: OffsetPageTable<'static>,
}

pub unsafe fn init(physical_memory_offest: PhysicalMemoryOffest) -> PageMapper {
    let level4_table = active_level4_table(&physical_memory_offest);

    PageMapper {
        _inner: OffsetPageTable::new(level4_table, physical_memory_offest.as_virt_addr()),
    }
}

unsafe fn active_level4_table(
    physical_memory_offest: &PhysicalMemoryOffest,
) -> &'static mut PageTable {
    let (level4_table_frame, _): (PhysFrame, Cr3Flags) = Cr3::read();
    let physical_address: PhysAddr = level4_table_frame.start_address();
    let virtual_address: VirtAddr = physical_memory_offest.as_virt_addr() + physical_address.as_u64();
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

// Minimal frame allocator backed directly by the bootloader's memory map.
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    // Start allocating from the first usable frame.
    pub fn init(memory_map: &'static MemoryMap) -> Self {
        Self {
            memory_map,
            next: 0,
        }
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        // Pick the `next` usable frame from the memory map.
        let frame = usable_frame(self.memory_map).nth(self.next);

        // Advance so the next allocation returns a different frame.
        self.next += 1;
        frame
    }
}
