use alloc::vec::Vec;
use custom_error::custom_error;

// library includes
use crate::iomem::IOBufChain;

custom_error! {pub DevQueueError
    BufferInvalid = "one of the supplied buffers was invalid",
    OutOfMemory = "the operation caused an out-of-memory condition",
    QueueFailure = "Unknown queue failer",
}

/// A device queue interface supporting enqueue/dequeueu
pub trait DevQueue {
    /**
     * Enqueues one or multiple IOBufChains into the queue that implements this trait.
     * This updates the descriptors of the queue accordingly, but the buffers may not yet
     * be made available for the device, and a flush() is required afterwards.
     *
     * The function returns buffer chains that have not been equeued to to limited available
     * space on the queue.
     *
     *  - bufs: a vector of buffers chains to be enqueued on the card
     */
    fn enqueue(&mut self, bufs: Vec<IOBufChain>) -> Result<Vec<IOBufChain>, DevQueueError>;

    /**
     * notifies the device that there have been new descriptors added to the queue
     * returns the number of buffers that have been handed over to the device
     */
    fn flush(&mut self) -> Result<usize, DevQueueError>;

    /**
     * Checks if new buffers can be enqueued and returns the number of available slots.
     * The returned count should reflect the actual available slots if the `exact` parameter
     * is true, otherwise non zero indicates there is at least one slot available.
     *
     *  - exact: flag indicating whether the exact amount should be calculated
     */
    fn can_enqueue(&self, exact: bool) -> Result<usize, DevQueueError>;

    /**
     * dequeues up to a number of `cnt` buffers from the queue. The buffers are returned
     * as the chains they were enqueued in.
     *
     *  - cnt: the maximum amount of buffers to process
     */
    fn dequeue(&mut self, cnt: usize) -> Result<Vec<IOBufChain>, DevQueueError>;

    /**
     * Checks if there are buffers ready to be dequeued and returns the count of processed
     * buffers.
     *
     *  - exact: flag indicating whether the exact amount should be calculated
     */
    fn can_dequeue(&mut self, exact: bool) -> Result<usize, DevQueueError>;
}
