use crate::serial_println;
use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::registers::control::{Cr3, Cr3Flags};
use x86_64::structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, Page, PageSize, PageTable, PageTableFlags, PhysFrame,
    Size4KiB,
};
use x86_64::{PhysAddr, VirtAddr};

pub struct PhysicalMemoryOffest(u64);

impl PhysicalMemoryOffest {
    pub const fn new(offset: u64) -> Self {
        Self(offset)
    }

    // Internally we still use `x86_64::VirtAddr`, but the wrapper keeps that
    // detail inside `memory.rs`. The offset itself is just an address in the
    // kernel's virtual address space.
    fn as_virt_addr(&self) -> VirtAddr {
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
    let level4_table = active_level4_table(&physical_memory_offest);

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
