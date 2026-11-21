use airborne_cot_relay::payload_compressor::deflate_compress;
use airborne_cot_relay::{
    CompressedPayload, CompressionAlgorithm, CompressorConfig, CotPayloadCompressor,
};

#[test]
fn decompress_enforces_max_payload_bytes_on_the_decoded_output() {
    let config = CompressorConfig::default().with_max_payload_bytes(1024);
    let comp = CotPayloadCompressor::new(config);
    let hostile = CompressedPayload {
        algorithm: CompressionAlgorithm::Deflate,
        uncompressed_len: 16,
        data: deflate_compress(&vec![0u8; 4_000_000]),
    };
    let wire = hostile.to_framed_bytes();
    assert!(wire.len() < 40_000);
    let parsed = CompressedPayload::from_framed_bytes(&wire).unwrap();
    let out = comp.decompress(&parsed).map(|bytes| bytes.len());
    assert!(
        out.is_err(),
        "limit 1024, header claims 16, decoded {out:?} bytes"
    );
}
