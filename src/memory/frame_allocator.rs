use crate::memory::usable_frame;
use bootloader::bootinfo::MemoryMap;
use x86_64::structures::paging::{FrameAllocator, PhysFrame, Size4KiB};

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
