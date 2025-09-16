// memory_manager.rs
// Memory management for N64's limited RAM

use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::ptr::NonNull;

#[cfg(test)]
use alloc::boxed::Box;

// Define memory regions
const HEAP_START: usize = 0x80300000; // Starting after N64's OS
const HEAP_SIZE: usize = crate::config::HEAP_SIZE_BYTES;

struct BumpArena {
    heap_start: usize,
    heap_end: usize,
    next_free: usize,
    checkpoints: [usize; 8],
    current_checkpoint: usize,
}

impl BumpArena {
    const fn new() -> Self {
        Self {
            heap_start: HEAP_START,
            heap_end: HEAP_START + HEAP_SIZE,
            next_free: HEAP_START,
            checkpoints: [0; 8],
            current_checkpoint: 0,
        }
    }

    fn alloc(&mut self, size: usize, align: usize) -> Option<NonNull<u8>> {
        let alloc_start = align_up(self.next_free, align);
        let alloc_end = alloc_start.checked_add(size)?;
        if alloc_end > self.heap_end {
            return None;
        }
        self.next_free = alloc_end;
        NonNull::new(alloc_start as *mut u8)
    }

    fn checkpoint(&mut self) -> usize {
        let idx = self.current_checkpoint;
        if idx < self.checkpoints.len() {
            self.checkpoints[idx] = self.next_free;
            self.current_checkpoint += 1;
            idx
        } else {
            self.checkpoints.len() - 1
        }
    }

    fn pop_checkpoint(&mut self) -> bool {
        if self.current_checkpoint == 0 {
            return false;
        }
        self.current_checkpoint -= 1;
        self.next_free = self.checkpoints[self.current_checkpoint];
        true
    }

    fn reset(&mut self) {
        self.next_free = self.heap_start;
        self.current_checkpoint = 0;
    }

    fn available_memory(&self) -> usize {
        self.heap_end - self.next_free
    }

    fn total_memory(&self) -> usize {
        self.heap_end - self.heap_start
    }

    fn used_memory(&self) -> usize {
        self.next_free - self.heap_start
    }
}

struct GlobalArena {
    arena: UnsafeCell<BumpArena>,
}

impl GlobalArena {
    const fn new() -> Self {
        Self {
            arena: UnsafeCell::new(BumpArena::new()),
        }
    }

    fn with_mut<R>(&self, f: impl FnOnce(&mut BumpArena) -> R) -> R {
        // Safety: single-threaded runtime; callers ensure no re-entrancy.
        let arena = unsafe { &mut *self.arena.get() };
        f(arena)
    }
}

unsafe impl Sync for GlobalArena {}

static GLOBAL_ARENA: GlobalArena = GlobalArena::new();

pub struct MemoryManager {
    arena: &'static GlobalArena,
}

impl MemoryManager {
    pub fn new() -> Self {
        Self {
            arena: &GLOBAL_ARENA,
        }
    }

    pub fn alloc(&mut self, size: usize, align: usize) -> Option<NonNull<u8>> {
        self.arena.with_mut(|arena| arena.alloc(size, align))
    }

    pub fn checkpoint(&mut self) -> usize {
        self.arena.with_mut(|arena| arena.checkpoint())
    }

    pub fn pop_checkpoint(&mut self) -> bool {
        self.arena.with_mut(|arena| arena.pop_checkpoint())
    }

    pub fn reset(&mut self) {
        self.arena.with_mut(|arena| arena.reset());
    }

    pub fn log_usage(&self, label: &str) {
        use crate::display;
        use alloc::format;

        let (used, total) = self
            .arena
            .with_mut(|arena| (arena.used_memory(), arena.total_memory()));
        let msg = format!("[mem] {}: used {} / {} bytes", label, used, total);
        display::print_line(&msg);
    }

    pub fn available_memory(&self) -> usize {
        self.arena.with_mut(|arena| arena.available_memory())
    }

    pub fn total_memory(&self) -> usize {
        self.arena.with_mut(|arena| arena.total_memory())
    }

    pub fn used_memory(&self) -> usize {
        self.arena.with_mut(|arena| arena.used_memory())
    }
}

pub unsafe fn init() -> MemoryManager {
    let mut mm = MemoryManager::new();
    mm.reset();
    mm
}

#[cfg(test)]
pub fn new_for_test() -> MemoryManager {
    let arena = Box::leak(Box::new(GlobalArena::new()));
    MemoryManager { arena }
}

fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator;

pub struct BumpAllocator;

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        GLOBAL_ARENA
            .with_mut(|arena| arena.alloc(layout.size(), layout.align()))
            .map_or(core::ptr::null_mut(), |nn| nn.as_ptr())
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // bump allocator does not free individual allocations
    }
}
