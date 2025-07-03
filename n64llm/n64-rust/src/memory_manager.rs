// memory_manager.rs
// Memory management for N64's limited RAM

use core::alloc::{GlobalAlloc, Layout};
use core::ptr::NonNull;

// Define memory regions
const HEAP_START: usize = 0x80300000; // Starting after N64's OS
const HEAP_SIZE: usize = 0x100000;    // 1MB heap

// Memory manager that handles allocation in the limited N64 memory
pub struct MemoryManager {
    heap_start: usize,
    heap_end: usize,
    next_free: usize,
    // Add checkpoint support for layer-based resets
    checkpoints: [usize; 8],
    current_checkpoint: usize,
}

impl MemoryManager {
    // Create a new MemoryManager
    pub fn new() -> Self {
        MemoryManager {
            heap_start: HEAP_START,
            heap_end: HEAP_START + HEAP_SIZE,
            next_free: HEAP_START,
            checkpoints: [0; 8],
            current_checkpoint: 0,
        }
    }
    
    // Allocate memory from the heap
    pub fn alloc(&mut self, size: usize, align: usize) -> Option<NonNull<u8>> {
        // Align the allocation address
        let alloc_start = align_up(self.next_free, align);
        let alloc_end = alloc_start.checked_add(size)?;
        
        if alloc_end <= self.heap_end {
            // We have enough space
            self.next_free = alloc_end;
            
            // Return pointer to allocated memory
            NonNull::new(alloc_start as *mut u8)
        } else {
            // Out of memory
            None
        }
    }
    
    // Free memory (simple bump allocator doesn't actually free individual allocations)
    pub fn dealloc(&mut self, _ptr: NonNull<u8>, _size: usize) {
        // In a bump allocator, we don't actually free individual allocations
    }
    
    // Create a checkpoint for the current allocation position
    pub fn checkpoint(&mut self) -> usize {
        let checkpoint_idx = self.current_checkpoint;
        if checkpoint_idx < self.checkpoints.len() {
            self.checkpoints[checkpoint_idx] = self.next_free;
            self.current_checkpoint += 1;
            checkpoint_idx
        } else {
            // If we're out of checkpoint slots, return the last one
            self.checkpoints.len() - 1
        }
    }
    
    // Restore to a previous checkpoint
    pub fn restore(&mut self, checkpoint_idx: usize) -> bool {
        if checkpoint_idx < self.current_checkpoint {
            self.next_free = self.checkpoints[checkpoint_idx];
            self.current_checkpoint = checkpoint_idx + 1;
            true
        } else {
            false
        }
    }

    // Restore to the most recent checkpoint and remove it
    pub fn pop_checkpoint(&mut self) -> bool {
        if self.current_checkpoint == 0 {
            return false;
        }
        self.current_checkpoint -= 1;
        self.next_free = self.checkpoints[self.current_checkpoint];
        true
    }
    
    // Reset all allocations (useful between inference steps)
    pub fn reset(&mut self) {
        self.next_free = self.heap_start;
        self.current_checkpoint = 0;
    }
    
    // Get available memory
    pub fn available_memory(&self) -> usize {
        self.heap_end - self.next_free
    }
    
    // Get total memory
    pub fn total_memory(&self) -> usize {
        self.heap_end - self.heap_start
    }
    
    // Get used memory
    pub fn used_memory(&self) -> usize {
        self.next_free - self.heap_start
    }

    // Log current memory usage with a label for debugging
    pub fn log_usage(&self, label: &str) {
        use alloc::format;
        use crate::display;

        let msg = format!(
            "[mem] {}: used {} / {} bytes",
            label,
            self.used_memory(),
            self.total_memory()
        );
        display::print_line(&msg);
    }
}

// Initialize the memory manager
pub unsafe fn init() -> MemoryManager {
    MemoryManager::new()
}

// Helper function to align addresses
fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

// Implement global allocator (required for Rust's allocation APIs)
#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator;

pub struct BumpAllocator;

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // This is a placeholder - would need proper thread-safe implementation
        static mut MEMORY_MANAGER: Option<MemoryManager> = None;
        
        if MEMORY_MANAGER.is_none() {
            MEMORY_MANAGER = Some(MemoryManager::new());
        }
        
        if let Some(ref mut mm) = MEMORY_MANAGER {
            match mm.alloc(layout.size(), layout.align()) {
                Some(ptr) => ptr.as_ptr(),
                None => core::ptr::null_mut(),
            }
        } else {
            core::ptr::null_mut()
        }
    }
    
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // No deallocation in bump allocator
    }
}