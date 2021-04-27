use alloc::alloc::{Allocator, Layout};
use alloc::collections::vec_deque::VecDeque;
use alloc::vec;
use alloc::vec::Vec;
use alloc::{alloc::AllocError, collections::TryReserveError};
use core::cmp;
use core::ptr::NonNull;

use custom_error::custom_error;
use x86::current::paging::{PAddr, VAddr, IOAddr};

// custom error for the IOMemory
custom_error! {pub IOMemError
    OutOfMemory = "reached out of memory",
    NotYetImplemented = "feature not yet implemented"
}

impl From<TryReserveError> for IOMemError {
    fn from(e: TryReserveError) -> Self {
        IOMemError::OutOfMemory
    }
}

///  TODO: get rid of this:
pub const KERNEL_BASE: u64 = 0x400000000000;

/// A trait to tag objects which a device needs to read or write over DMA.
pub trait DmaObject {
    fn paddr(&self) -> PAddr {
        PAddr::from(&*self as *const Self as *const () as u64) - PAddr::from(KERNEL_BASE)
    }

    fn vaddr(&self) -> VAddr {
        VAddr::from(&*self as *const Self as *const () as usize)
    }

    fn ioaddr(&self) -> IOAddr {
        IOAddr::from(self.paddr())
    }
}

/// Allocator that backs memory for IOBufs.
pub struct IOMemAllocator {
    /// Layout established for this allocator.
    layout: Layout,
}

/// IOMemAllocator Implementation
impl IOMemAllocator {
    fn new(layout: Layout) -> Result<IOMemAllocator, IOMemError> {
        Ok(IOMemAllocator { layout: layout })
    }
}

unsafe impl Allocator for IOMemAllocator {
    /// Allocates IO memory.
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        // get the size to be allocated as a multiple of the initialized layout of the allocator
        let sz = ((layout.size() + layout.size() - 1) / self.layout.size()) * self.layout.size();
        let align = cmp::max(self.layout.align(), layout.align());

        // construct the new layout for allocation
        let alloc_layout = Layout::from_size_align(sz, align).expect("Layout was invalid.");

        unsafe {
            // do the actual allocation
            // TODO: refer to the OS allocator
            let ptr: *mut u8 = alloc::alloc::alloc_zeroed(alloc_layout);
            if ptr.is_null() {
                return Err(AllocError);
            }

            // wrap in in NonNull, remove option type
            let ptr_nonnull = NonNull::new(ptr).unwrap();

            // construct the NonNull slice for the return
            Ok(NonNull::slice_from_raw_parts(ptr_nonnull, sz))
        }
    }

    /// Deallocates the previously allocated IO memory.
    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        // XXX: check the layout matches the allocator here?
        let buf = ptr.as_ptr();
        // TODO: refer to the OS allocator
        alloc::alloc::dealloc(buf, layout);
    }
}

/// Represents an IO buffer (data handed to/from device).
pub struct IOBuf {
    buf: Vec<u8, IOMemAllocator>,
}

impl IOBuf {
    pub fn new(layout: Layout) -> Result<IOBuf, IOMemError> {
        // get the aligned buffer length
        // get the layouf for the allocation
        let allocator = IOMemAllocator::new(layout);
        let buf: Vec<u8, IOMemAllocator> = Vec::with_capacity_in(layout.size(), allocator.unwrap());

        Ok(IOBuf { buf: buf })
    }

    /// Removes all buffer contents.
    pub fn clear(&mut self) {
        self.buf.clear();
    }

    /// Copy data from `src` into a given `offset` of the `IOBuf`.
    pub fn copy_in_at(&mut self, offset: usize, src: &[u8]) -> Result<usize, IOMemError> {
        // Currently we do not allow extending the buffer:
        let remaining_capacity = self.buf.capacity() - offset;
        let cnt = cmp::min(remaining_capacity, src.len());
        self.buf.resize(offset + cnt, 0);

        // copy the slice
        self.buf[offset..offset + cnt].copy_from_slice(&src[0..cnt]);

        Ok(cnt)
    }

    /// Copy raw data of size `len` into the buffer.
    pub fn copy_in(&mut self, src: &[u8]) -> Result<usize, IOMemError> {
        self.copy_in_at(0, src)
    }

    /// Copy data out of the IOBuf, starting at a given `offset` into `dst`.
    pub fn copy_out_at(&self, offset: usize, dst: &mut [u8]) -> Result<usize, IOMemError> {
        // of the offset is outside of the length of the vector then we
        if offset >= self.buf.len() {
            return Ok(0);
        }

        let cnt = cmp::min(self.buf.len() - offset, dst.len());
        // copy the slice
        dst[0..cnt].copy_from_slice(&self.buf[offset..offset + cnt]);
        Ok(cnt)
    }

    /// Copy the data (starting at 0) to `dst` slice.
    pub fn copy_out(&self, dst: &mut [u8]) -> Result<usize, IOMemError> {
        self.copy_out_at(0, dst)
    }

    /// Get a IOBuf contents as slice.
    pub fn as_slice(&self) -> Result<&[u8], IOMemError> {
        Ok(self.buf.as_slice())
    }

    /// Get a IOBuf contents as mutable slice.
    pub fn as_mut_slice(&mut self) -> Result<&mut [u8], IOMemError> {
        Ok(self.buf.as_mut_slice())
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }
}

impl DmaObject for IOBuf {
    /// Address of the IOBuf in main memory.
    fn paddr(&self) -> PAddr {
        PAddr::from(self.buf.as_ptr() as u64 - KERNEL_BASE)
    }

    /// Virtual address this buffer can be access by software.
    fn vaddr(&self) -> VAddr {
        VAddr::from(self.buf.as_ptr() as u64)
    }
}

/// A pool of buffers IOBuf's with the same size and for the same the device.
pub struct IOBufPool {
    /// Pool of buffers
    pool: Vec<IOBuf>,
    /// The allocator used for new buffers
    allocator: IOMemAllocator,
    /// The allocation layout of the buffers
    layout: Layout,
}

impl IOBufPool {
    pub fn new(len: usize, align: usize) -> Result<IOBufPool, IOMemError> {
        let layout = Layout::from_size_align(len, align).expect("Layout was invalid.");
        let allocator = IOMemAllocator::new(layout);

        Ok(IOBufPool {
            pool: vec![],
            allocator: allocator.unwrap(),
            layout: layout,
        })
    }

    pub fn get_buf(&mut self) -> Result<IOBuf, IOMemError> {
        match self.pool.pop() {
            Some(x) => Ok(x),
            None => IOBuf::new(self.layout),
        }
    }

    pub fn put_buf(&mut self, buf: IOBuf) {
        self.pool.push(buf)
    }
}
/// An IO buffer.
pub struct IOBufChain {
    /// Corresponding queue ID.
    pub qsidx: usize,

    /// XXX
    pub pidx: usize,

    /// Set by `enqueue`
    pub ipi_new_pidx: usize,

    /// Flags (to be used by device driver).
    pub flags: u32,

    /// The `IOBuf` fragments
    pub segments: VecDeque<IOBuf>,
}

impl IOBufChain {
    pub fn new(
        qsidx: usize,
        pidx: usize,
        flags: u32,
        len: usize,
    ) -> Result<IOBufChain, IOMemError> {
        let mut vd = VecDeque::new();
        vd.try_reserve_exact(len)?;

        Ok(IOBufChain {
            pidx,
            qsidx,
            flags,
            ipi_new_pidx: 0,
            segments: vd,
        })
    }
}
