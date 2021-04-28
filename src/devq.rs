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
    /// Enqueues an IOBufChain into the queue that implements this trait. This
    /// updates the descriptors of the queue accordingly, but the buffers may
    /// not yet be made available for the device, and a flush() is required
    /// afterwards.
    ///
    /// The function returns the buffer chain if there was not enough space to
    /// enqueue all buffers in this chain. (e.g., due to limited available space
    /// on the queue).
    ///
    /// # Arguments
    /// - bufs: a vector of buffers chains to be enqueued on the card
    ///
    /// # Return
    /// - On success: returns nothing
    /// - On error, in case there was no space left on the device ring, it may
    ///   return the entire IOBufChain back to the client. The implementor
    ///   should ensure not to partially enqueue an IOBufChain in this situation
    ///   by checking for available space up-front.
    fn enqueue(&mut self, bufs: IOBufChain) -> Result<(), IOBufChain>;

    /// Notifies the device that there have been new descriptors added to the
    /// queue.
    ///
    /// # Returns
    /// Returns the number of IOBufChains that have been handed to the device.
    fn flush(&mut self) -> Result<usize, DevQueueError>;

    /// Checks if new buffers can be enqueued and returns the number of
    /// available slots. The returned count should reflect the actual available
    /// slots if the `exact` parameter is true, otherwise non zero indicates
    /// there is at least one slot available.
    ///
    /// # Arguments
    ///  - `how_many_seg`: Indicate how many segments (of one or multiple
    ///    IOBufChain) the client wants to enqueue.
    ///
    /// # Returns
    ///  - true if we can enqueue that many segemnts in the device ring
    ///  - false if we don't have enough space in the device ring
    fn can_enqueue(&self, how_many_seg: usize) -> bool;

    /// Dequeues a previously enqueued IOBufChain from the queue which has been
    /// processed. The buffers shall be returned back in FIFO order.
    ///
    /// # Returns
    /// - On success, one (processed) IOBufChain
    /// - A DevQueueError, for example if there is no IOBufChain ready to
    ///   dequeue.
    fn dequeue(&mut self) -> Result<IOBufChain, DevQueueError>;

    /// Checks if there are buffers ready to be dequeued and returns the count
    /// of processed buffers.
    ///
    /// # Arguments
    ///  - exact: flag indicating whether the exact amount should be calculated.
    ///    if this is false the implementation may process less items, but
    ///    should try to return at least 1 if there is something to be dequed.
    ///
    /// # Returns
    /// - The number of IOBufChains that are ready to be dequeued (using
    ///   `dequeue`).
    fn can_dequeue(&mut self, exact: bool) -> usize;
}
