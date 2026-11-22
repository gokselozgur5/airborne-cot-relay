//! Lock-free atomic ring buffer for Cursor-on-Target (CoT) telemetry events.

use std::cell::UnsafeCell;
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicUsize, Ordering};

use sightline_cot::cot::CotEvent;

/// Cell slot within the atomic ring buffer.
struct RingCell<T> {
    sequence: AtomicUsize,
    value: UnsafeCell<MaybeUninit<T>>,
}

impl<T> RingCell<T> {
    fn new(seq: usize) -> Self {
        Self {
            sequence: AtomicUsize::new(seq),
            value: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }
}

/// Lock-free bounded multi-producer multi-consumer atomic ring buffer.
pub struct AtomicRingBuffer<T> {
    buffer: Box<[RingCell<T>]>,
    mask: usize,
    capacity: usize,
    enqueue_pos: AtomicUsize,
    dequeue_pos: AtomicUsize,
}

unsafe impl<T: Send> Send for AtomicRingBuffer<T> {}
unsafe impl<T: Send> Sync for AtomicRingBuffer<T> {}

impl<T> AtomicRingBuffer<T> {
    /// Creates a new atomic ring buffer with rounded power-of-two capacity.
    pub fn new(capacity: usize) -> Self {
        let cap = capacity.max(2).next_power_of_two();
        let mut cells = Vec::with_capacity(cap);
        for i in 0..cap {
            cells.push(RingCell::new(i));
        }
        Self {
            buffer: cells.into_boxed_slice(),
            mask: cap - 1,
            capacity: cap,
            enqueue_pos: AtomicUsize::new(0),
            dequeue_pos: AtomicUsize::new(0),
        }
    }

    /// Returns the buffer capacity.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns the approximate number of items currently in the buffer.
    pub fn len(&self) -> usize {
        let head = self.dequeue_pos.load(Ordering::Relaxed);
        let tail = self.enqueue_pos.load(Ordering::Relaxed);
        tail.saturating_sub(head).min(self.capacity)
    }

    /// Returns true if the buffer has no queued items.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns true if the buffer is currently at full capacity.
    pub fn is_full(&self) -> bool {
        self.len() >= self.capacity
    }

    /// Enqueues an item into the ring buffer. Returns Err(item) if full.
    pub fn push(&self, data: T) -> Result<(), T> {
        let mut pos = self.enqueue_pos.load(Ordering::Relaxed);
        loop {
            let cell = &self.buffer[pos & self.mask];
            let seq = cell.sequence.load(Ordering::Acquire);
            let diff = seq as isize - pos as isize;
            if diff == 0 {
                if self
                    .enqueue_pos
                    .compare_exchange_weak(pos, pos + 1, Ordering::AcqRel, Ordering::Relaxed)
                    .is_ok()
                {
                    unsafe {
                        (*cell.value.get()).write(data);
                    }
                    cell.sequence.store(pos + 1, Ordering::Release);
                    return Ok(());
                }
                pos = self.enqueue_pos.load(Ordering::Relaxed);
            } else if diff < 0 {
                return Err(data);
            } else {
                pos = self.enqueue_pos.load(Ordering::Relaxed);
            }
        }
    }

    /// Dequeues an item from the ring buffer. Returns None if empty.
    pub fn pop(&self) -> Option<T> {
        let mut pos = self.dequeue_pos.load(Ordering::Relaxed);
        loop {
            let cell = &self.buffer[pos & self.mask];
            let seq = cell.sequence.load(Ordering::Acquire);
            let diff = seq as isize - (pos + 1) as isize;
            if diff == 0 {
                if self
                    .dequeue_pos
                    .compare_exchange_weak(pos, pos + 1, Ordering::AcqRel, Ordering::Relaxed)
                    .is_ok()
                {
                    let val = unsafe { (*cell.value.get()).assume_init_read() };
                    cell.sequence.store(pos + self.mask + 1, Ordering::Release);
                    return Some(val);
                }
                pos = self.dequeue_pos.load(Ordering::Relaxed);
            } else if diff < 0 {
                return None;
            } else {
                pos = self.dequeue_pos.load(Ordering::Relaxed);
            }
        }
    }

    /// Drains all available items into a vector.
    pub fn drain(&self) -> Vec<T> {
        let mut items = Vec::new();
        while let Some(item) = self.pop() {
            items.push(item);
        }
        items
    }

    /// Discards all items currently in the buffer.
    pub fn clear(&self) {
        while self.pop().is_some() {}
    }
}

impl<T> Drop for AtomicRingBuffer<T> {
    fn drop(&mut self) {
        while self.pop().is_some() {}
    }
}

/// Lock-free circular telemetry buffer dedicated to Cursor-on-Target (CoT) events.
pub struct CotRingBuffer {
    inner: AtomicRingBuffer<CotEvent>,
    dropped_count: AtomicUsize,
}

impl CotRingBuffer {
    /// Creates a new CotRingBuffer with the requested capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: AtomicRingBuffer::new(capacity),
            dropped_count: AtomicUsize::new(0),
        }
    }

    /// Returns the buffer capacity.
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// Returns current number of CoT events queued.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns true if the buffer has no events queued.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Enqueues a CoT event. Returns Err(event) if full.
    #[allow(clippy::result_large_err)]
    pub fn push(&self, event: CotEvent) -> Result<(), CotEvent> {
        self.inner.push(event)
    }

    /// Dequeues the next available CoT event.
    pub fn pop(&self) -> Option<CotEvent> {
        self.inner.pop()
    }

    /// Returns true if the buffer is at capacity.
    pub fn is_full(&self) -> bool {
        self.inner.is_full()
    }

    /// Enqueues an event, dropping the oldest event if full.
    pub fn push_overwrite(&self, mut event: CotEvent) {
        while let Err(ev) = self.inner.push(event) {
            if self.inner.pop().is_some() {
                self.dropped_count.fetch_add(1, Ordering::Relaxed);
            }
            event = ev;
        }
    }

    /// Returns cumulative dropped event count from buffer overflow.
    pub fn dropped_count(&self) -> usize {
        self.dropped_count.load(Ordering::Relaxed)
    }

    /// Drains all available CoT events.
    pub fn drain_all(&self) -> Vec<CotEvent> {
        self.inner.drain()
    }

    /// Clears all events in the buffer.
    pub fn clear(&self) {
        self.inner.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atomic_ring_buffer_basic_push_pop() {
        let buf = AtomicRingBuffer::new(4);
        assert_eq!(buf.capacity(), 4);
        assert!(buf.is_empty());
        assert_eq!(buf.push(10), Ok(()));
        assert_eq!(buf.push(20), Ok(()));
        assert_eq!(buf.len(), 2);
        assert_eq!(buf.pop(), Some(10));
        assert_eq!(buf.pop(), Some(20));
        assert_eq!(buf.pop(), None);
        assert!(buf.is_empty());
    }

    #[test]
    fn test_atomic_ring_buffer_full_and_drain() {
        let buf = AtomicRingBuffer::new(2);
        assert_eq!(buf.push(1), Ok(()));
        assert_eq!(buf.push(2), Ok(()));
        assert!(buf.is_full());
        assert_eq!(buf.push(3), Err(3));
        let drained = buf.drain();
        assert_eq!(drained, vec![1, 2]);
        assert!(buf.is_empty());
        assert_eq!(buf.push(4), Ok(()));
        buf.clear();
        assert!(buf.is_empty());
    }

    fn make_test_cot(uid: &str) -> CotEvent {
        CotEvent {
            uid: uid.to_string(),
            cot_type: "a-f-G-U-C".to_string(),
            how: "m-g".to_string(),
            time_ms: 1000,
            stale_ms: 2000,
            lat_deg: 35.0,
            lon_deg: 42.0,
            hae_m: 100.0,
            ce_m: 5.0,
            le_m: 10.0,
            course_deg: 180.0,
            speed_mps: 25.0,
        }
    }

    #[test]
    fn test_cot_ring_buffer_operations() {
        let cot_buf = CotRingBuffer::new(2);
        assert_eq!(cot_buf.capacity(), 2);
        assert!(cot_buf.push(make_test_cot("cot-1")).is_ok());
        assert!(cot_buf.push(make_test_cot("cot-2")).is_ok());
        assert!(cot_buf.is_full());
        let popped = cot_buf.pop().unwrap();
        assert_eq!(popped.uid, "cot-1");
        assert_eq!(cot_buf.len(), 1);
        cot_buf.clear();
        assert!(cot_buf.is_empty());
    }

    #[test]
    fn test_cot_ring_buffer_push_overwrite() {
        let cot_buf = CotRingBuffer::new(2);
        cot_buf.push_overwrite(make_test_cot("cot-1"));
        cot_buf.push_overwrite(make_test_cot("cot-2"));
        cot_buf.push_overwrite(make_test_cot("cot-3"));
        assert_eq!(cot_buf.dropped_count(), 1);
        let events = cot_buf.drain_all();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].uid, "cot-2");
        assert_eq!(events[1].uid, "cot-3");
    }
}
