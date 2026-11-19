use airborne_cot_relay::fragment_reassembly::{Fragment, FragmentHeader, FragmentReassembler};
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
struct Counting;
static LIVE: AtomicUsize = AtomicUsize::new(0);
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        LIVE.fetch_add(layout.size(), Relaxed);
        System.alloc(layout)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size(), Relaxed);
        System.dealloc(ptr, layout)
    }
}
#[global_allocator]
static ALLOC: Counting = Counting;
#[test]
fn header_only_packets_do_not_reserve_megabytes() {
    let mut reassembler = FragmentReassembler::default();
    let before = LIVE.load(Relaxed);
    for id in 0..256u32 {
        let header = FragmentHeader::new(id, 0, u16::MAX).unwrap();
        let packet = Fragment::new(header, Vec::new()).encode();
        reassembler.process_packet(&packet, 0).unwrap();
    }
    let held = LIVE.load(Relaxed).saturating_sub(before);
    assert!(held < 1 << 20, "256 packets of 8 bytes pinned {held} bytes");
}
