use crate::memory::{active_level4_table, BootInfoFrameAllocator};
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageTableFlags, PhysFrame, Size4KiB,
};
use x86_64::VirtAddr;

pub struct PhysicalMemoryOffest(u64);

impl PhysicalMemoryOffest {
    pub const fn new(offset: u64) -> Self {
        Self(offset)
    }

    // Internally we still use `x86_64::VirtAddr`, but the wrapper keeps that
    // detail inside `memory.rs`. The offset itself is just an address in the
    // kernel's virtual address space.
    pub(crate) fn as_virt_addr(&self) -> VirtAddr {
        VirtAddr::new(self.0)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

// Facade type for the active paging state.
// `main.rs` can hold this value without depending on `OffsetPageTable` directly.
//
// A "mapper" is the object responsible for manipulating page-table mappings:
// it knows how to translate "map this virtual page to that physical frame"
// into concrete page-table writes.
pub struct PageMapper {
    _inner: OffsetPageTable<'static>,
}

// Build a mapper for the page tables that the bootloader already installed.
// The offset comes from `BootInfo::physical_memory_offset`.
//
// We do not create a fresh paging hierarchy here. We reuse the one that is
// already active when the bootloader enters the kernel.
pub unsafe fn init(physical_memory_offest: &PhysicalMemoryOffest) -> PageMapper {
    let level4_table = active_level4_table(physical_memory_offest);

    PageMapper {
        _inner: OffsetPageTable::new(level4_table, physical_memory_offest.as_virt_addr()),
    }
}

impl PageMapper {
    // Map one 4 KiB virtual page to one 4 KiB physical frame.
    //
    // Paging works at page granularity, so even if later code writes only a
    // single `u64`, the CPU still needs a page-table entry for the whole page
    // containing that address.
    pub fn map_page(
        &mut self,
        virt_addr_raw: u64,
        boot_info_frame_allocator: &mut BootInfoFrameAllocator,
    ) {
        let virt_addr: VirtAddr = VirtAddr::new(virt_addr_raw);
        let page: Page<Size4KiB> = Page::containing_address(virt_addr);

        let frame: PhysFrame = boot_info_frame_allocator
            .allocate_frame()
            .expect("failed to allocate physical frame");

        // `PRESENT` means the page is valid and may participate in address
        // translation. `WRITABLE` means writes through this mapping are allowed.
        // Without these flags, the CPU would either reject the mapping or treat
        // it as read-only.
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        let flush = unsafe {
            // `map_to` updates the page-table hierarchy so that `page` resolves
            // to `frame`. If intermediate page tables are missing, it uses the
            // frame allocator to create them.
            self._inner
                .map_to(page, frame, flags, boot_info_frame_allocator)
                .expect("map_to failed")
        };

        // The CPU caches recent virtual->physical translations in the TLB
        // (Translation Lookaside Buffer). After changing the page tables, we
        // must invalidate the stale cached entry so future accesses use the new
        // mapping we just installed.
        flush.flush();
    }
}
