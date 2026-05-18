use core::alloc::{GlobalAlloc, Layout};
use linked_list_allocator::LockedHeap;
use x86_64::structures::paging::{PageSize, Size4KiB};
use crate::memory::{BootInfoFrameAllocator, PageMapper};

#[global_allocator]
static ALLOCATOR: KernelHeapAllocator = KernelHeapAllocator::empty();

const HEAP_START: u64 = 0x_0000_5555_0000_0000;
const HEAP_BYTE_SIZE: usize = 100 * 1024; // 100KiB

pub fn init_heap(
    frame_allocator: &mut BootInfoFrameAllocator,
    page_mapper: &mut PageMapper,
) {
    let mut virt_addr_raw: u64 = HEAP_START;
    let n_frames: usize = HEAP_BYTE_SIZE / Size4KiB::SIZE as usize;
    for _ in 0..n_frames {
        page_mapper.map_page(virt_addr_raw, frame_allocator);
        virt_addr_raw += Size4KiB::SIZE;
    }

    unsafe {
        ALLOCATOR.init(HEAP_START, HEAP_BYTE_SIZE);
    }
}

struct KernelHeapAllocator {
    _allocator: LockedHeap,
}

impl KernelHeapAllocator {
    pub const fn empty() -> Self {
        Self {
            _allocator: LockedHeap::empty(),
        }
    }

    unsafe fn init(&self, heap_start: u64, heap_size: usize) {
        self._allocator.lock().init(heap_start as *mut u8, heap_size);
    }
}

unsafe impl GlobalAlloc for KernelHeapAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self._allocator.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self._allocator.dealloc(ptr, layout)
    }
}