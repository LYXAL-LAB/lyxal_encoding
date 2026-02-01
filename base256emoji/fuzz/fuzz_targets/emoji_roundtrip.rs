#![no_main]
use libfuzzer_sys::fuzz_target;
use lyxal_base256emoji::{encode, decode};

fuzz_target!(|data: &[u8]| {
    let encoded = encode(data);
    let decoded = decode(&encoded).expect("Fuzzing failed: decode should always succeed after encode");
    assert_eq!(data, decoded.as_slice());
});
