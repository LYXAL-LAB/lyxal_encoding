#![no_main]
use libfuzzer_sys::fuzz_target;
use base_x;

fuzz_target!(|data: &[u8]| {
    let alphabet = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    
    // Test Alloc path
    if let Ok(encoded) = base_x::encode(alphabet, data) {
        let decoded = base_x::decode(alphabet, &encoded).expect("Round-trip failed");
        assert_eq!(data, decoded);
    }

    // Test No-Alloc path (with smaller buffer for performance)
    let mut out_buf = [0u8; 1024];
    if let Ok(len) = base_x::encode_to_buffer(alphabet.as_bytes(), data, &mut out_buf) {
        let mut dec_buf = [0u8; 1024];
        let dec_len = base_x::decode_to_buffer(alphabet.as_bytes(), core::str::from_utf8(&out_buf[..len]).unwrap(), &mut dec_buf).expect("Round-trip failed in no_alloc");
        assert_eq!(data, &dec_buf[..dec_len]);
    }
});
