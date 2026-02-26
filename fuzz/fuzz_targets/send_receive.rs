//! If we panic!, we lose.
//!
//! ```not_rust
//! cargo +nightly fuzz run send_receive -- -max_len=1022
//! ```
//! 1022 to leave room for '\r\n' (Lines) and '#' (Delimiter).
//! The read/write buffers are 1024 bytes.

#![no_main]

use std::{
    error::Error,
    fmt::{Debug, Display},
};

use arbitrary::Unstructured;
use embedded_io_adapters::tokio_1::FromTokio;
use framez::{
    codec::{
        delimiter::Delimiter,
        lines::{Lines, StrLines},
    },
    decode::{DecodeError, Decoder},
    encode::Encoder,
    next, FramedRead, FramedWrite,
};
use libfuzzer_sys::fuzz_target;
use tokio::runtime::Runtime;

fuzz_target!(|data: &[u8]| {
    Runtime::new()
        .expect("Runtime must build")
        .block_on(async move {
            let delimiter = Unstructured::new(data)
                .arbitrary::<u8>()
                .expect("Failed to generate delimiter");

            fuzz(
                data,
                Delimiter::new(delimiter),
                Delimiter::new(delimiter),
                |data| (!data.contains(&delimiter)).then_some(data).ok_or(()),
            )
            .await
            .unwrap();

            fuzz(data, Lines::new(), Lines::new(), |data| {
                (!data.contains(&b'\n')).then_some(data).ok_or(())
            })
            .await
            .unwrap();

            fuzz(data, StrLines::new(), StrLines::new(), |data| {
                (!data.contains(&b'\n')).then_some(data).ok_or(())?;

                str::from_utf8(data).map_err(|_| ())
            })
            .await
            .unwrap();
        });
});

// Note: Bytes can not be fuzzed like this
async fn fuzz<'data, D, E, F, T>(
    data: &'data [u8],
    encoder: E,
    decoder: D,
    map: F,
) -> Result<(), Box<dyn Error>>
where
    E: Encoder<T> + 'static,
    <E as Encoder<T>>::Error: Error + Display + 'static,
    D: for<'buf> Decoder<'buf> + 'static,
    for<'buf> <D as Decoder<'buf>>::Item: 'buf + Debug + PartialEq<T>,
    <D as DecodeError>::Error: Error + Display + 'static,
    F: FnOnce(&'data [u8]) -> Result<T, ()>,
    T: 'data + Clone + Debug + PartialEq,
{
    // If we can not create an item from the data, we do not have to bother with the rest.
    let item = match map(data) {
        Ok(item) => item,
        Err(_) => return Ok(()),
    };

    let (read, write) = tokio::io::duplex(32);

    let item_clone = item.clone();
    let read_buf = &mut [0u8; 1024];
    let mut framed_read = FramedRead::new(decoder, FromTokio::new(read), read_buf);

    let reader = async move {
        match next!(framed_read) {
            Some(read_item) => {
                let read_item = read_item?;

                assert_eq!(read_item, item_clone);

                Ok::<(), Box<dyn Error>>(())
            }
            None => panic!("Should receive a frame"),
        }
    };

    let write_buf = &mut [0u8; 1024];
    let mut framed_write = FramedWrite::new(encoder, FromTokio::new(write), write_buf);

    let writer = async move {
        framed_write.send(item).await?;

        Ok::<(), Box<dyn Error>>(())
    };

    let (reader_result, writer_result) = tokio::join!(reader, writer);

    reader_result?;
    writer_result?;

    Ok(())
}
