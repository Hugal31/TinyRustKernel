#![feature(allocator_api)]
#![feature(align_offset)]
#![feature(ptr_offset_from)]
#![cfg_attr(not(test), no_std)]

extern crate spin;

use core::alloc::{Alloc, AllocErr, GlobalAlloc, Layout};
use core::fmt;
use core::marker::PhantomData;
use core::mem::{size_of, transmute};
use core::num::NonZeroUsize;
use core::ptr::{self, NonNull};

use spin::Mutex;

pub struct KAllocator {
    /// Start of the usable memory
    memory_start: *mut u8,
    /// End of the usable memory
    memory_end: *mut u8,
}

unsafe impl Send for KAllocator {}

impl KAllocator {
    pub const fn invalid() -> KAllocator {
        KAllocator {
            memory_start: ptr::null_mut(),
            memory_end: ptr::null_mut(),
        }
    }

    pub unsafe fn new(memory_start: NonNull<u8>, memory_end: NonNull<u8>) -> KAllocator {
        let memory_start = memory_start.as_ptr();
        let memory_end = memory_end.as_ptr();
        let memory_size = memory_end.offset_from(memory_start) as usize;
        assert!(
            memory_size > size_of::<Block>(),
            "The memory should be at least {} bytes long",
            size_of::<Block>()
        );

        let free_block = memory_start as *mut Block;
        *free_block = Block::unused(memory_size - size_of::<Block>());

        KAllocator {
            memory_start,
            memory_end,
        }
    }

    fn blocks(&self) -> impl Iterator<Item = &'static mut Block> {
        BlockIterator::new(
            unsafe { &mut *(self.memory_start as *mut Block) },
            self.memory_end,
        )
    }

    fn blocks_for(&self, layout: Layout) -> impl Iterator<Item = &'static mut Block> {
        self.blocks()
            .filter(move |b| !b.used && b.size() >= layout.size())
    }

    fn previous_block(&self, block: &Block) -> Option<&'static mut Block> {
        self.blocks().find(|b| unsafe { b.next_block().is(block) })
    }

    fn is_in_memory(&self, block: &Block) -> bool {
        block.address() != self.memory_end
    }
}

unsafe impl Alloc for KAllocator {
    unsafe fn alloc(&mut self, layout: Layout) -> Result<NonNull<u8>, AllocErr> {
        let mut blocks_iterator = self.blocks_for(layout);
        let mut block = blocks_iterator.next().ok_or(AllocErr)?;

        // Handle alignment
        let mut align_offset = block.data_address().align_offset(layout.align());
        while align_offset != 0 {
            // If there is enough space, create a free block at the beginning and use the rest,
            // which is aligned
            if block.size() >= align_offset + layout.align() - size_of::<Block>() + layout.size() {
                let previous_size = block.size();
                block.set_size(align_offset + layout.align() - size_of::<Block>());
                block = block.next_block_mut();
                block
                    .set_size(previous_size - (align_offset + layout.align())); // I remove  "- size_of::<Block>()", now it works
                debug_assert_eq!(0, block.data_address().align_offset(layout.align()));
                break;
            } else {
                // Else, search another block.
                block = blocks_iterator.next().ok_or(AllocErr)?;
                align_offset = block.data_address().align_offset(layout.align());
            }
        }

        block.used = true;

        // Split if possible
        // TODO Handle alignment
        if block.size() > layout.size() + size_of::<Block>() {
            // Reduce the size of the block and create a new one
            let previous_size = block.size();
            block.set_size(layout.size());

            let next_block = block.next_block_mut();
            next_block.set_size(previous_size - layout.size() - size_of::<Block>());
            next_block.used = false;
        }

        Ok(NonNull::new_unchecked(block.data_address()))
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, _layout: Layout) {
        let block = Block::from_data_address(ptr.as_ptr());
        block.used = false;

        // Merge next block
        let next_block = block.next_block();
        if self.is_in_memory(next_block) && !next_block.used {
            block.set_size(block.size() + next_block.size_with_header());
        }

        // Merge previous block
        if let Some(previous_block) = self.previous_block(block) {
            if !previous_block.used {
                previous_block.set_size(previous_block.size() + block.size_with_header());
            }
        }
    }
}

pub struct GlobalKalloc {
    allocator: Mutex<KAllocator>,
}

impl GlobalKalloc {
    pub const fn invalid() -> GlobalKalloc {
        GlobalKalloc {
            allocator: Mutex::new(KAllocator::invalid()),
        }
    }

    pub fn set_memory_bounds(&self, memory_start: NonNull<u8>, memory_end: NonNull<u8>) {
        *self.allocator.lock() = unsafe { KAllocator::new(memory_start, memory_end) }
    }
}

unsafe impl GlobalAlloc for GlobalKalloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.allocator
            .lock()
            .alloc(layout)
            .expect("Should allocate")
            .as_ptr()
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.allocator
            .lock()
            .dealloc(NonNull::new(ptr).unwrap(), layout)
    }
}

impl fmt::Debug for KAllocator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for block in self.blocks() {
            write!(
                f,
                "[{}:{}]",
                if block.used { "USED" } else { "FREE" },
                block.size()
            )?;
        }

        Ok(())
    }
}

struct Block {
    size: NonZeroUsize,
    used: bool,
}

impl Block {
    fn unused(size: usize) -> Block {
        Block {
            size: unsafe { NonZeroUsize::new_unchecked(size + size_of::<Self>()) },
            used: false,
        }
    }

    unsafe fn from_data_address(data: *mut u8) -> &'static mut Block {
        &mut *(data.sub(size_of::<Self>()) as *mut Block)
    }

    /// Unsafe because the caller needs to check the next block is in the usable memory
    unsafe fn next_block(&self) -> &'static Block {
        &*(self.address().add(self.size_with_header()) as *const Block)
    }

    unsafe fn next_block_mut(&mut self) -> &'static mut Block {
        &mut *(self.address().add(self.size_with_header()) as *mut Block)
    }

    #[inline]
    fn is(&self, other: &Block) -> bool {
        self.address() == other.address()
    }

    fn set_size(&mut self, size: usize) {
        self.size = unsafe { NonZeroUsize::new_unchecked(size + size_of::<Self>()) };
    }

    #[inline]
    fn size(&self) -> usize {
        self.size.get() - size_of::<Self>()
    }

    #[inline]
    fn size_with_header(&self) -> usize {
        self.size.get()
    }

    #[inline]
    fn address(&self) -> *mut u8 {
        unsafe { transmute(self) }
    }

    #[inline]
    fn data_address(&self) -> *mut u8 {
        unsafe { self.address().add(size_of::<Self>()) }
    }
}

struct BlockIterator<'a> {
    block: *mut Block,
    memory_end: *mut u8,
    marker: PhantomData<&'a Block>,
}

impl<'a> BlockIterator<'a> {
    fn new(block: &'a mut Block, memory_end: *mut u8) -> Self {
        BlockIterator {
            block: block as *mut Block,
            memory_end,
            marker: PhantomData,
        }
    }

    fn is_at_end(&self) -> bool {
        unsafe { (*self.block).address() == self.memory_end }
    }
}

impl<'a> Iterator for BlockIterator<'a> {
    type Item = &'a mut Block;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_at_end() {
            return None;
        }

        unsafe {
            let current_block = self.block.as_mut().unwrap();
            self.block = current_block
                .address()
                .add(current_block.size_with_header()) as *mut Block;

            Some(current_block)
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn assert_block_count(allocator: &mut KAllocator, expected_count: usize) {
        assert_eq!(
            expected_count,
            allocator.blocks().count(),
            "Should have {} block at the init. Now: {:?}",
            expected_count,
            allocator
        );
    }

    fn new_allocator(memory: &mut [u8]) -> KAllocator {
        unsafe {
            KAllocator::new(
                NonNull::new_unchecked(&mut memory[0] as *mut u8),
                NonNull::new_unchecked((&mut memory[0] as *mut u8).add(memory.len())),
            )
        }
    }

    #[test]
    fn test_invalid() {
        unsafe {
            let mut allocator = KAllocator::invalid();

            assert!(allocator.alloc(Layout::new::<u32>()).is_err());
        }
    }

    #[test]
    fn test_one_alloc() {
        let mut memory = [0u8; 1024];

        unsafe {
            let memory_start = &mut memory[0] as *mut u8;
            let mut allocator = new_allocator(&mut memory);

            assert_block_count(&mut allocator, 1);

            let layout = Layout::from_size_align(1, 1).unwrap();
            let ptr = allocator.alloc(layout).expect("Should allocate");
            assert_eq!(memory_start.add(size_of::<Block>()), ptr.as_ptr());
            assert_block_count(&mut allocator, 2);

            allocator.dealloc(ptr, layout);
            assert_block_count(&mut allocator, 1);
        };
    }

    #[test]
    fn test_too_big_alloc() {
        let mut memory = [0u8; 1024];

        unsafe {
            let mut allocator = new_allocator(&mut memory);

            let layout = Layout::from_size_align(2048, 1).unwrap();
            assert!(allocator.alloc(layout).is_err());
        };
    }

    #[test]
    fn test_merge_alloc() {
        let mut memory = [0u8; 1024];
        let memory_start = &mut memory[0] as *mut u8;

        unsafe {
            let mut allocator = new_allocator(&mut memory);

            let layout = Layout::from_size_align(256, 1).unwrap();
            let ptr1 = allocator.alloc(layout).expect("Should allocate");
            let ptr2 = allocator.alloc(layout).expect("Should allocate");
            let ptr3 = allocator.alloc(layout).expect("Should allocate");

            assert_block_count(&mut allocator, 4);

            assert_eq!(memory_start.add(size_of::<Block>()), ptr1.as_ptr());
            assert_eq!(
                memory_start
                    .add(size_of::<Block>())
                    .add(layout.size())
                    .add(size_of::<Block>()),
                ptr2.as_ptr()
            );
            assert_eq!(
                memory_start
                    .add(size_of::<Block>())
                    .add(layout.size())
                    .add(size_of::<Block>())
                    .add(layout.size())
                    .add(size_of::<Block>()),
                ptr3.as_ptr()
            );

            allocator.dealloc(ptr1, layout);
            allocator.dealloc(ptr3, layout);
            allocator.dealloc(ptr2, layout);

            assert_block_count(&mut allocator, 1);
        };
    }

    #[test]
    fn test_alloc_in_middle() {
        let mut memory = [0u8; 1024];

        unsafe {
            let mut allocator = new_allocator(&mut memory);

            let layout = Layout::from_size_align(256, 1).unwrap();
            let _ptr1 = allocator.alloc(layout).expect("Should allocate");
            let ptr2 = allocator.alloc(layout).expect("Should allocate");
            let _ptr3 = allocator.alloc(layout).expect("Should allocate");

            allocator.dealloc(ptr2, layout);

            let ptr4 = allocator
                .alloc(Layout::from_size_align(254, 1).unwrap())
                .expect("Should allocate");
            assert_eq!(ptr2, ptr4);
        }
    }

    #[test]
    fn test_align() {
        let mut memory = [0u8; 1024];

        unsafe {
            let mut allocator = new_allocator(&mut memory);

            let layout = Layout::from_size_align(256, 256).unwrap();
            let ptr = allocator.alloc(layout).expect("Should allocate");
            assert_eq!(0, ptr.as_ptr().align_offset(256));
        }
    }

    #[test]
    fn test_bug1() {
        let mut memory = vec![0u8; 8192];

        unsafe {
            let mut allocator = new_allocator(&mut memory[0..8192]);

            let ptr1 = allocator.alloc(Layout::from_size_align(36, 4).unwrap()).expect("Should allocate");
            let _ptr2 = allocator.alloc(Layout::from_size_align(43, 4096).unwrap()).expect("Should allocate");
            let _ptr3 = allocator.alloc(Layout::from_size_align(12, 8).unwrap()).expect("Should allocate");

            allocator.dealloc(ptr1, Layout::from_size_align(36, 4).unwrap());
        }
    }
}
