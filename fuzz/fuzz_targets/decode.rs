//! If we panic!, we lose.
//!
//! ```not_rust
//! cargo +nightly fuzz run decode
//! ```

#![no_main]

use arbitrary::Unstructured;
use framez::{
    codec::{
        bytes::Bytes,
        delimiter::Delimiter,
        lines::{Lines, StrLines},
    },
    decode::Decoder,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let buf = &mut [0_u8; 64];
    let data = &mut std::vec::Vec::from(data);

    let delimiter = Unstructured::new(data)
        .arbitrary::<u8>()
        .expect("Failed to generate delimiter");

    let mut codec = Delimiter::new(delimiter);
    let _ = codec.decode(data).expect("Must be Infallible");

    let mut codec = Bytes::new();
    let _ = codec.decode(buf).expect("Must be Infallible");

    let mut codec = Lines::new();
    let _ = codec.decode(buf).expect("Must be Infallible");

    let mut codec = StrLines::new();

    match core::str::from_utf8(data) {
        Ok(_) => {
            let _ = codec.decode(buf).expect("Must be Infallible");
        }
        Err(_) => {
            let _ = codec.decode(buf);
        }
    }
});
