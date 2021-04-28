use custom_error::custom_error;

extern crate smoltcp;



// library includes
use crate::devq::DevQueue;
use crate::iomem::{IOBufChain, IOBufPool};

// smoltcp includes
use smoltcp::phy::{self, Device, DeviceCapabilities};
use smoltcp::time::Instant;
use smoltcp::Result;

/// define the maximum packet size supported
const MAX_PACKET_SZ: usize = 2048;

// custom error functions
custom_error! {pub DevQueuePhyError
    PhyFailure = "Unknown phy failer"
}

/// a smoltcp phy implementation wrapping a DevQueue
struct DevQueuePhy {
    rx_q: DevQueue,
    tx_q: DevQueue,
    pool: IOBufPool,
}

impl DevQueuePhy {
    fn new(rx_q: DevQueue, tx_q: DevQueue) -> core::result::Result<DevQueuePhy, DevQueuePhyError> {
        let pool = IOBufPool::new(MAX_PACKET_SZ, MAX_PACKET_SZ);
        match pool {
            Ok(p) => Ok(DevQueuePhy {
                rx_q: rx_q,
                tx_q: tx_q,
                pool: p,
            }),
            Err(p) => Err(PhyFailure)
        }
    }
}

impl smoltcp::phy::Device for DevQueuePhy {

    type RxToken = RxPacket<'a>;
    type TxToken = TxPacket<'a>;

   /**
     * obtains a receive buffer along a side a send buffer for replies (e.g., ping...)
     */
    fn receive(&'a mut self) -> Option<(Self::RxToken, Self::TxToken)> {
        // check if there is any packet available in the receive queue
        let numdeq =
            match self.rx_q.can_dequeue(false) {
                Ok(num) => num,
                Err(_)  => return None
            };

        if  numdeq > 0 {
            // get the packet, for now just one
            let packet = self.rx_q.dequeue(1);

            // enqueue another buffer for future receives, TODO: maybe need to enqueue more than one
            let bufs = IOBufChain::new(0, 0, 0, 1);
            bufs.append(self.pool.get_buf())
            self.rx_q.enqueue(bufs)

            // construct the RX token
            let rx_token = RxPacket<'a>(packet, self.rx_q, self.pool);

            // get an empty TX token from the pool...
            // TODO: make sure we can actually send something!
            let iobuf = IOBufChain::new(0, 0, 0, 1);
            iobuf.append(self.pool.get_buf())
            let tx_token = TxPacket<'a>(iobuf, self.tx_q, self.pool);

            Some(rx_token, tx_token)
        } else {
            None
        }
    }

    /**
     * obtains/allocates an empty end buffer
     */
    fn transmit(&'a mut self) -> Option<Self::TxToken> {
        // see if there is something to dequeue
        let numdeq =
            match self.rx_q.can_dequeue(false) {
                Ok(num) => num,
                Err(_)  => 0
            };

        let packet = if numdeq > 0 {
            self.rx_q.dequeue(1)
            } else {
                let iobuf = IOBufChain::new(0, 0, 0, 1);
                iobuf.append(self.pool.get_buf());
                iobuf
            }

        // get an empty TX token from the pool
        Some(TxPacket<'a>(packet, self.tx_q, self.pool))
    }

    /**
     * the device capabilities (e.g., checksum offloading etc...)
     */
    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = 1536;
        caps.max_burst_size = Some(1);
        caps
    }
}


/// smolnet RxToken

/// smolnet TxToken
struct RxPacket<'a> {
    iobuf: IOBufChain,
    txq : DevQueue,
    pool: IOBufPool
}

impl RxPacket<'a> {
    fn new(iobuf: IOBufChain, txq : DevQueue, pool: IOBufPool) -> RxPacket<'a> {
        RxPacket<'a> {
            iobuf: iobuf,
            txq : txq,
            pool: pool
        }
    }
}

impl<'a> phy::RxToken for RxPacket<'a> {
    fn consume<R, F>(mut self, _timestamp: Instant, f: F) -> Result<R>
        where F: FnOnce(&mut [u8]) -> Result<R>
    {
        // XXX: not sure here if the buffer actually needs to be copied here...
        let result = f(&mut self.iobuf);
        //println!("rx called");

        // we can drop the IOBufChain here.
        self.pool.put_buf(self.iobuf);
        result
    }
}


/// smolnet TxToken
struct TxPacket<'a> {
    iobuf: IOBufChain,
    txq : DevQueue,
    pool: IOBufPool
}

impl TxPacket<'a> {
    fn new(iobuf: IOBufChain, txq : DevQueue, pool: IOBufPool) -> TxPacket<'a> {
        TxPacket<'a> {
            iobuf: iobuf,
            txq : txq,
            pool: pool
        }
    }
}

/// implements the TxToken trait for the TxPacket
impl<'a> phy::TxToken for TxPacket<'a> {
    fn consume<R, F>(self, _timestamp: Instant, len: usize, f: F) -> Result<R>
        where F: FnOnce(&mut [u8]) -> Result<R>
    {
        println!("tx called {}", len);

        // let the network stack write into the packet
        let result = f(&mut self.iobuf[0..len]);

        // TODO: send packet out, this passes ownership of the IOBufChain to the device
        // XXX: can we guarantee that there is space in the queue?
        self.txq.enqueue(self.iobuf);
        self.txq.flush();

        // references dropped...
        result
    }
}

/// Drop trait to return a dropped Tx token back to the pool
impl<'a> Drop for TxPacket<'a> {
    /// gets called when the TxPacket gets dropped in the stack
    fn drop(&mut self) {
        // TODO: return the buffer back to the pool.
        // self.pool.put_buf(self.iobuf);
    }
}


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
