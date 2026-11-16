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
                if self.enqueue_pos.compare_exchange_weak(
                    pos, pos + 1, Ordering::AcqRel, Ordering::Relaxed,
                ).is_ok() {
                    unsafe { (*cell.value.get()).write(data); }
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
}
