use alloc::alloc::{Allocator, Layout};
use alloc::collections::vec_deque::VecDeque;
use alloc::vec;
use alloc::vec::Vec;
use alloc::{alloc::AllocError, collections::TryReserveError};
use core::cmp;
use core::ops::Index;
use core::ptr::NonNull;

use log::info;

use custom_error::custom_error;
use x86::current::paging::{IOAddr, PAddr, VAddr};

// custom error for the IOMemory
custom_error! {pub IOMemError
    OutOfMemory = "reached out of memory",
    NotYetImplemented = "feature not yet implemented"
}

impl From<TryReserveError> for IOMemError {
    fn from(_e: TryReserveError) -> Self {
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
        IOAddr::from(self.paddr().as_u64())
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

#[derive(Debug)]
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
        let mut iobuf = IOBuf { buf };
        // call expand here to make sure the buffer has the full size
        iobuf.expand();
        info!("IOBuf: new buffer of size {}!",iobuf.capacity());
        Ok(iobuf)
    }

    /// Fill buffer with as many 0 as capacity allows.
    pub fn expand(&mut self) {
        self.buf.resize(self.buf.capacity(), 0);
    }

    pub fn truncate(&mut self, new_len: usize) {
        self.buf.truncate(new_len)
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
    pub fn as_slice(&self) -> &[u8] {
        self.buf.as_slice()
    }

    /// Get a IOBuf contents as mutable slice.
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        self.buf.as_mut_slice()
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }
}

/// implementation for the index operator [] on IOBuf
impl Index<usize> for IOBuf {
    /// The returned type after indexing.
    type Output = u8;

    /// Performs the indexing (`container[index]`) operation.
    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        &self.buf[index]
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
    _allocator: IOMemAllocator,
    /// The allocation layout of the buffers
    layout: Layout,
}

impl IOBufPool {
    pub fn new(len: usize, align: usize) -> Result<IOBufPool, IOMemError> {
        let layout = Layout::from_size_align(len, align).expect("Layout was invalid.");
        let allocator = IOMemAllocator::new(layout);

        Ok(IOBufPool {
            pool: vec![],
            _allocator: allocator.unwrap(),
            layout: layout,
        })
    }

    pub fn get_buf(&mut self) -> Result<IOBuf, IOMemError> {
        if self.pool.len() > 0 {
            let mut buf = self.pool.pop().expect("should have a buffer here");
            buf.expand();
            Ok(buf)
        } else {
            IOBuf::new(self.layout)
        }
    }

    pub fn put_buf(&mut self, buf: IOBuf) {
        self.pool.push(buf)
    }
}

#[derive(Debug)]
/// An IO buffer.
pub struct IOBufChain {
    /// Completion queue index (set by driver),
    /// TODO: remove once no longer necessary?
    cqidx: usize,

    /// Check sum flags (set by driver on rx)
    pub csum_flags: u32,

    /// Checksum data (set by driver on rx)
    pub csum_data: u32,

    /// VLAN tag (set by device driver on rx)
    pub vtag: Option<u32>,

    /// Flags (to be used by device driver).
    pub flags: u32,

    /// Flow ID for RSS
    pub rss_flow_id: Option<usize>,

    /// RSS type
    pub rss_type: u32,

    /// The `IOBuf` fragments
    pub segments: VecDeque<IOBuf>,
}

impl IOBufChain {
    pub fn new(flags: u32, len: usize) -> Result<IOBufChain, IOMemError> {
        let mut vd = VecDeque::new();
        vd.try_reserve_exact(len)?;

        Ok(IOBufChain {
            cqidx: 0,
            flags,
            csum_flags: 0,
            csum_data: 0,
            vtag: None,
            rss_flow_id: None,
            rss_type: 0,
            segments: vd,
        })
    }

    /// Set meta-data provided by the driver
    pub fn set_meta_data(
        &mut self,
        total_len: usize,
        segments: usize,
        cqidx: usize,
        rss_flow_id: Option<usize>,
        rsstype: u32,
    ) {
        self.cqidx = cqidx;
        self.rss_flow_id = rss_flow_id;
        self.rss_type = rsstype;

        // Truncate unused segments to zero
        // count unused segments
        let mut remaining_bytes = total_len;
        let mut unused_segments = 0;
        for seg in self.segments.iter_mut() {
            if remaining_bytes == 0 {
                seg.truncate(0); // unused segment
                unused_segments += 1;
            }
            remaining_bytes -= seg.len();
        }

        assert_eq!(
            segments,
            self.segments.len() - unused_segments,
            "#Segments match"
        );
        assert_eq!(
            total_len,
            self.segments.iter().map(|s| s.len()).sum(),
            "total_len matches"
        );
    }

    pub fn append(&mut self, buf: IOBuf) {
        self.segments.push_back(buf);
    }
}

/// implementation for the index operator [] on IOBuf
impl Index<usize> for IOBufChain {
    /// The returned type after indexing.
    type Output = u8;

    /// Performs the indexing (`container[index]`) operation.
    fn index(&self, index: usize) -> &Self::Output {
        let mut cidx = index;
        let nseg = self.segments.len();
        for i in 0..nseg {
            let seglen = self.segments[i].len();
            if index < seglen {
                return &self.segments[i][cidx];
            }
            cidx -= seglen;
        }
        // error here?
        &self.segments[0][0]
    }
}
