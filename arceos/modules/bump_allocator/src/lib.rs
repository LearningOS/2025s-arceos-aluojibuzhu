#![no_std]

extern crate alloc;
pub const GRANULARITY: usize = core::mem::size_of::<usize>() * 4;
use allocator::{BaseAllocator, ByteAllocator, PageAllocator};
use alloc::vec::Vec;
const ALIGN:usize=8;
use allocator::AllocResult;
use log::debug;
use core::alloc::Layout;
use core::fmt::Debug;
//use core::alloc::AllocError;
use core::ptr::NonNull;
const MAX_BLOCKS: usize = 8; 
/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
///
#[derive(Debug, Clone, Copy)] 
pub struct EarlyAllocator{
    block:[Option<Blockmemory>;MAX_BLOCKS],
    currentid:Option<usize>,
    total_bytes: usize,
    used_bytes: usize,
    available_bytes:usize,
}

#[derive(Debug, Clone, Copy)] 
pub struct Blockmemory{
    start:usize,
    end:usize,
    byte_end:usize,
    page_start:usize,
    total_bytes: usize,
    used_bytes: usize,
    available_bytes:usize,
}

impl EarlyAllocator {
    pub const fn new()->Self{
        Self { 
            block: [None;MAX_BLOCKS], 
            currentid: None,
            total_bytes: 1,
            used_bytes: 1,
            available_bytes:1, 
        }
    }
}

impl BaseAllocator for EarlyAllocator {
    fn init(&mut self, start: usize, size: usize){
    debug!("the start is {} size is {} ",start,size);

    self.block[0]=Some(Blockmemory{
           start:start,
           end:start+size,
           byte_end:start,
           page_start:start+size,
           total_bytes: size,
           used_bytes: 0,
           available_bytes:size,
    });
    debug!("init structis {:?}",self.block[0]);
        self.currentid=Some(0);
        self.total_bytes+=size;
        self.available_bytes+=size;
    }

    /// Add a free memory region to the allocator.
    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult{
        let mut index=0;
        for bl in self.block.iter_mut(){
            if bl.is_none(){
                *bl=Some(Blockmemory{
                    start:start,
                    end:start+size,
                    byte_end:start,
                    page_start:start+size,
                    total_bytes: size,
                    used_bytes: 0,
                    available_bytes:size,
             })
            }
            index+=1;
        }
        debug!("push structis {:?}",self.block[self.block.len()-1]);
            self.currentid=Some(self.block.len()-1);
            self.total_bytes+=size;
            self.available_bytes+=size;
        Ok(())
    }
}

impl ByteAllocator for EarlyAllocator {
        /// Allocate memory with the given size (in bytes) and alignment.
        fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>>{
            debug!("self id {:?}",self.block);
            if let Some(block)=self.block.iter_mut().find(|x| {x.as_ref().unwrap().available_bytes>=layout.size()}){
                let start=block.as_mut().unwrap().byte_end+1;
                block.as_mut().unwrap().available_bytes-=layout.size();
                block.as_mut().unwrap().used_bytes+=layout.size();
                block.as_mut().unwrap().byte_end+=layout.size();
                self.available_bytes-=layout.size();
                self.used_bytes+=layout.size();
                return Ok(NonNull::new(start as *mut u8).unwrap());
            }
            unsafe { Ok(NonNull::new_unchecked(0 as *mut u8)) }
        }
        /// Deallocate memory at the given position, size, and alignment.
        fn dealloc(&mut self, pos: NonNull<u8>, layout: Layout){
            let start =pos.as_ptr() as usize;
            let block=self.block.iter_mut().find(|x| {x.as_ref().unwrap().byte_end>=start}).unwrap();
            block.as_mut().unwrap().available_bytes+=layout.size();
            block.as_mut().unwrap().used_bytes-=layout.size();
            block.as_mut().unwrap().byte_end-=layout.size();

            self.available_bytes+=layout.size();
            self.used_bytes-=layout.size();
        }

    
        /// Returns total memory size in bytes.
        fn total_bytes(&self) -> usize{
            self.total_bytes
        }
    
        /// Returns allocated memory size in bytes.
        fn used_bytes(&self) -> usize{
            self.used_bytes
        }
    
        /// Returns available memory size in bytes.
        fn available_bytes(&self) -> usize{
            self.available_bytes
        }
}

impl PageAllocator for EarlyAllocator {
        /// The size of a memory page.
    const PAGE_SIZE: usize = 0x1000;

    /// Allocate contiguous memory pages with given count and alignment.
    fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> AllocResult<usize>{
        Ok(0)
    }

    /// Deallocate contiguous memory pages with given position and count.
    fn dealloc_pages(&mut self, pos: usize, num_pages: usize){

    }

    /// Returns the total number of memory pages.
    fn total_pages(&self) -> usize{
        0
    }

    /// Returns the number of allocated memory pages.
    fn used_pages(&self) -> usize{
        0
    }

    /// Returns the number of available memory pages.
    fn available_pages(&self) -> usize{
        0
    }
}

