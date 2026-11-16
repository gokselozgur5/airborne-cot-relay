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
