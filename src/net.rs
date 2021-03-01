use custom_error::custom_error;
pub struct PktInfo {
    pub qsidx: usize,
}

impl PktInfo {
    pub fn segments(&self) -> usize {
        0
    }
}

pub struct RxdInfo {}

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
