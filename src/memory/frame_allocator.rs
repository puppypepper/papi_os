use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::PhysAddr;
use x86_64::structures::paging::{FrameAllocator, PageSize, PhysFrame, Size4KiB};

// Minimal frame allocator backed directly by the bootloader's memory map.
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    // Start allocating from the first usable frame.
    // Safety: constructing multiple allocators from the same memory map would
    // allow the same physical frame to be handed out more than once.
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        Self {
            memory_map,
            next: 0,
        }
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
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        // Pick the `next` usable frame from the memory map.
        let frame = BootInfoFrameAllocator::usable_frame(self.memory_map).nth(self.next);

        // Advance so the next allocation returns a different frame.
        self.next += 1;
        frame
    }
}
