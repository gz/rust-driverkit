use custom_error::custom_error;

// library includes
use crate::iomem::IOBufChain;

custom_error! {pub DevQueueError
    BufferInvalid = "one of the supplied buffers was invalid",
    OutOfMemory = "the operation caused an out-of-memory condition",
    QueueFull = "the queue was full. Can't enqueue more buffers.",
    QueueEmpty = "the queue was empty. Nothing to deuqueue.",
    QueueFailure = "Unknown queue failure",
}

/// A device queue interface supporting enqueue/dequeueu
pub trait DevQueue {
    /// Enqueues an IOBufChain into the queue that implements this
    /// trait. This updates the descriptors of the queue accordingly, but the
    /// buffers may not yet be made available for the device, and a flush() is
    /// required afterwards.
    ///
    /// The function returns the buffer chain if there was not enough space to enqueue all
    /// buffers in this chain. (e.g., due to limited available space on the queue).
    ///
    /// # Arguments
    /// - bufs: a vector of buffers chains to be enqueued on the card
    fn enqueue(&mut self, bufs: IOBufChain) -> Result<(), IOBufChain>;

    /**
     * notifies the device that there have been new descriptors added to the queue
     * returns the number of buffers that have been handed over to the device
     */
    fn flush(&mut self, txqid: usize, pidx: usize) -> Result<usize, DevQueueError>;

    /**
     * Checks if new buffers can be enqueued and returns the number of available slots.
     * The returned count should reflect the actual available slots if the `exact` parameter
     * is true, otherwise non zero indicates there is at least one slot available.
     *
     *  - exact: flag indicating whether the exact amount should be calculated
     */
    fn can_enqueue(&self, exact: bool) -> Result<usize, DevQueueError>;

    /**
     * dequeues a previously enqueued buffer chain from the queue. The buffers are returned
     * as the chains they were enqueued in.
     */
    fn dequeue(&mut self) -> Result<IOBufChain, DevQueueError>;

    /**
     * Checks if there are buffers ready to be dequeued and returns the count of processed
     * buffers.
     *
     *  - exact: flag indicating whether the exact amount should be calculated
     */
    fn can_dequeue(&mut self, exact: bool) -> Result<usize, DevQueueError>;
}
