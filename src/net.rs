use alloc::vec::Vec;

use custom_error::custom_error;

pub struct RxdInfo {}

/*
pub struct PktInfo {
    pub qsidx: usize,
    pub flags: u32,

    /// bus_dma_segment_t
    /// A machine-dependent type that describes individual	DMA segments.
    /// It	contains the following fields:
    ///
    /// bus_addr_t	     ds_addr;
    /// bus_size_t	     ds_len;
    ///
    /// The ds_addr field contains	the device visible address of the DMA
    /// segment, and ds_len contains the length of	the DMA	segment.  Al-
    /// though the	DMA segments returned by a mapping call	will adhere to
    /// all restrictions necessary	for a successful DMA operation,	some
    /// conversion	(e.g. a	conversion from	host byte order	to the de-
    /// vice's byte order)	is almost always required when presenting seg-
    /// ment information to the device.
    pub segments: Vec<(u64, u32)>,
}

impl PktInfo {
    pub fn nsegs(&self) -> usize {
        self.segments.len()
    }

    pub fn pidx(&self) -> usize {
        0
    }
}


custom_error! {pub TxError
    Unknown = "Unknown error",
}

custom_error! {pub RxError
    Unknown = "Unknown error",
}

/// Device Dependent Transmit and Receive Functions
///
/// Inspired by FreeBSD's iflibtxrx
pub trait TxRx {
    fn txd_encap(&mut self, pi: PktInfo) -> Result<(), TxError>;

    fn txd_flush(&mut self, qid: u16);

    fn txd_credits_update(&mut self, qid: u16, clear: bool) -> Result<(), TxError>;

    fn isc_rxd_available(&mut self, qsid: u16, cidx: u32) -> Result<(), RxError>;

    fn rxd_refill(&mut self, qsid: u16, flid: u8, pidx: u32, paddrs: &[u64], vaddrs: &[u64]);

    fn rxd_flush(&mut self, qsid: u16, flid: u8, pidx: u32);

    fn rxd_pkt_get(&mut self, ri: RxdInfo) -> Result<(), RxError>;
}
*/
