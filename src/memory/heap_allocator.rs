use core::alloc::{GlobalAlloc, Layout};
use linked_list_allocator::LockedHeap;
use x86_64::structures::paging::{PageSize, Size4KiB};
use crate::memory::{BootInfoFrameAllocator, PageMapper};

// Register the kernel heap allocator as Rust's global allocator.
//
// Important:
// `empty()` does not mean the heap is ready to use. It only creates the static
// allocator object early enough for the compiler and `alloc` crate to know
// which allocator type to call later. The real heap arena is provided in
// `init_heap(...)` after we map heap pages into the kernel's virtual address
// space.
#[global_allocator]
static ALLOCATOR: KernelHeapAllocator = KernelHeapAllocator::empty();

// Reserve one fixed virtual range for the kernel heap.
//
// This is a virtual address, not a physical one. `init_heap(...)` will later
// back this range with real physical frames, one 4 KiB page at a time.
const HEAP_START: u64 = 0x_0000_5555_0000_0000;
const HEAP_BYTE_SIZE: usize = 100 * 1024; // 100KiB

pub fn init_heap(
    frame_allocator: &mut BootInfoFrameAllocator,
    page_mapper: &mut PageMapper,
) {
    // The heap allocator expets one contiguous virtual range that it can treat
    // as its arena. So first we make every 4KiB page in that range valid by
    // mapping it to a fresh physical frame.
    let mut virt_addr_raw: u64 = HEAP_START;
    let n_frames: usize = HEAP_BYTE_SIZE / Size4KiB::SIZE as usize;
    for _ in 0..n_frames {
        page_mapper.map_page(virt_addr_raw, frame_allocator);
        virt_addr_raw += Size4KiB::SIZE;
    }

    unsafe {
        // After the backing pages exist, hand the whole contiguous virtual
        // range to the heap allocator. From this point on, `Box`, `Vec`, and
        // other `alloc` types may obtain memory from this arena.
        ALLOCATOR.init(HEAP_START, HEAP_BYTE_SIZE);
    }
}

// Facade type around the concrete heap implementation.
//
// Rust's `#[global_allocator]` attribute requires a type implementing
// `GlobalAlloc`, but that does not need to be `LockedHeap` itself. We wrap the
// concrete allocator here so `linked_list_allocator` stays an internal detail
// of the memory subsystem.
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
        // `LockedHeap` manages a raw memory arena. We pass the start address of
        // the mapped heap region plus its byte size so it can build its free
        // list over that range.
        self._allocator.lock().init(heap_start as *mut u8, heap_size);
    }
}

unsafe impl GlobalAlloc for KernelHeapAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Delegate the actual allocation policy to `LockedHeap`. The facade's
        // job is only to satisfy Rust's global allocator interface while
        // keeping the concrete allocator type private to this module.
        self._allocator.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self._allocator.dealloc(ptr, layout)
    }
}
