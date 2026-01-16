/// https://doc.rust-lang.org/core/alloc/trait.GlobalAlloc.html
use spin::Mutex;
use core::alloc::{GlobalAlloc, Layout};

pub struct BumpAllocator {
    heap_start: usize,      /// Start address of the heap
    heap_end: usize,        /// End address of the heap
    next: usize,            /// Next free address
    allocations: usize,     /// Number of active allocations
}

/// Empty Allocator
impl BumpAllocator {
    pub const fn new() -> Self {
        BumpAllocator {
            heap_start: 0,
            heap_end: 0,
            next: 0,
            allocations: 0,
        }
    }

    /// Initializes the allocator with a given memory range
    /// Safety
    /// It’s unsafe because the caller must guarantee that the address is valid.
    pub unsafe fn init(&mut self, heap_start: usize, heap_size: usize) {
        self.heap_start = heap_start;
        self.heap_end = heap_start + heap_size;
        self.next = heap_start;
    }
}

/// Wrapper around an allocator to provide thread-safe access
pub struct Locked<X> {
    inner: Mutex<X>,
}

/// Thread-safe wrapper implementation
impl<X> Locked<X> {
    pub const fn new(inner: X) -> Self {
        Locked {
            inner: Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<X> {
        self.inner.lock()
    }
}

/// Global allocator implementation for the locked bump allocator
/// Safety
/// The methods must uphold the global allocator contract.
unsafe impl GlobalAlloc for Locked<BumpAllocator> {
    /// Allocation method
    /// Safety
    /// The caller must ensure that the layout is valid.
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut allocator = self.lock();

        let alloc_start = allocator.next;

        /// Align the allocation start address
        let alignment = layout.align();
        let alloc_start_aligned = (alloc_start + alignment - 1) & !(alignment - 1);
        
        /// Calculate the end address of the allocation
        let alloc_end = match alloc_start_aligned.checked_add(layout.size()) {
            Some(end) => end,
            None => return core::ptr::null_mut(),
        };

        /// Check if the allocation fits within the heap bounds
        if alloc_end > allocator.heap_end {
            core::ptr::null_mut()
        } else {
            allocator.next = alloc_end;
            allocator.allocations += 1;
            alloc_start_aligned as *mut u8
        }
    }

    /// Deallocation method
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        let mut allocator = self.lock();
        allocator.allocations -= 1;

        if allocator.allocations == 0 {
            allocator.next = allocator.heap_start;
        }
    }
}
