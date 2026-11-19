use airborne_cot_relay::payload_compressor::{CompressorConfig, CotPayloadCompressor};

fn pseudo_random_payload(len: usize) -> Vec<u8> {
    let mut x = 1u64;
    (0..len)
        .map(|_| {
            x = x
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (x >> 33) as u8
        })
        .collect()
}

#[test]
fn snappy_round_trips_incompressible_payload_over_64k() {
    let data = pseudo_random_payload(70_000);
    let mut comp = CotPayloadCompressor::new(CompressorConfig::default());
    let packed = comp
        .compress(&data)
        .expect("70 kB is below max_payload_bytes");
    let out = comp.decompress(&packed);
    assert_eq!(out.as_ref(), Ok(&data));
}
