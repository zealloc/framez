//! Any delimiter codecs for encoding and decoding bytes.

use core::convert::Infallible;

use crate::{
    decode::{DecodeError, Decoder},
    encode::Encoder,
};

/// A codec that decodes bytes ending with a `delimiter` into bytes and encodes bytes into bytes ending with a `delimiter`.
///
/// # Note
///
/// This codec tracks progress using an internal state of the underlying buffer, and it must not be used across multiple framing sessions.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct Delimiter {
    /// The delimiter to search for.
    delimiter: u8,
    /// The number of bytes of the slice that have been seen so far.
    seen: usize,
}

impl Delimiter {
    /// Creates a new [`Delimiter`] with the given `delimiter`.
    #[inline]
    pub const fn new(delimiter: u8) -> Self {
        Self { delimiter, seen: 0 }
    }

    /// Returns the delimiter to search for.
    #[inline]
    pub const fn delimiter(&self) -> u8 {
        self.delimiter
    }
}

impl DecodeError for Delimiter {
    type Error = Infallible;
}

impl<'buf> Decoder<'buf> for Delimiter {
    type Item = &'buf [u8];

    fn decode(&mut self, src: &'buf mut [u8]) -> Result<Option<(Self::Item, usize)>, Self::Error> {
        if src.is_empty() {
            return Ok(None);
        }

        while self.seen < src.len() {
            if src[self.seen] == self.delimiter {
                let bytes = &src[..self.seen];
                let item = (bytes, self.seen + 1);

                self.seen = 0;

                return Ok(Some(item));
            }

            self.seen += 1;
        }

        Ok(None)
    }
}

/// Error returned by [`Delimiter::encode`].
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DelimiterEncodeError {
    /// The input buffer is too small to fit the encoded bytes.
    BufferTooSmall,
}

impl core::fmt::Display for DelimiterEncodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            DelimiterEncodeError::BufferTooSmall => write!(f, "buffer too small"),
        }
    }
}

impl core::error::Error for DelimiterEncodeError {}

impl Encoder<&[u8]> for Delimiter {
    type Error = DelimiterEncodeError;

    fn encode(&mut self, item: &[u8], dst: &mut [u8]) -> Result<usize, Self::Error> {
        let size = item.len() + 1;

        if dst.len() < size {
            return Err(DelimiterEncodeError::BufferTooSmall);
        }

        dst[..item.len()].copy_from_slice(item);
        dst[item.len()..size].copy_from_slice(&[self.delimiter]);

        Ok(size)
    }
}

#[cfg(test)]
mod test {
    use std::vec::Vec;

    use futures::{SinkExt, StreamExt, pin_mut};
    use tokio::io::AsyncWriteExt;

    use crate::{
        ReadError,
        tests::{framed_read, init_tracing, sink_stream},
    };

    use super::*;

    #[tokio::test]
    async fn framed_read() {
        init_tracing();

        // cspell: disable
        let items: &[&[u8]] = &[
            b"jh asjd#ppppppppppppppp#",
            b"k hb#jsjuwjal kadj#jsadhjiu#w",
            b"#jal kadjjsadhjiuwqens #",
            b"nd ",
            b"yxxcjajsdi#askdn as",
            b"jdasd#iouqw es",
            b"sd#k",
        ];
        // cspell: enable

        let decoder = Delimiter::new(b'#');

        let expected: &[&[u8]] = &[];
        framed_read!(items, expected, decoder, 1, BufferTooSmall);
        framed_read!(items, expected, decoder, 1, 1, BufferTooSmall);
        framed_read!(items, expected, decoder, 1, 2, BufferTooSmall);
        framed_read!(items, expected, decoder, 1, 4, BufferTooSmall);

        framed_read!(items, expected, decoder, 2, BufferTooSmall);
        framed_read!(items, expected, decoder, 2, 1, BufferTooSmall);
        framed_read!(items, expected, decoder, 2, 2, BufferTooSmall);
        framed_read!(items, expected, decoder, 2, 4, BufferTooSmall);

        framed_read!(items, expected, decoder, 4, BufferTooSmall);
        framed_read!(items, expected, decoder, 4, 1, BufferTooSmall);
        framed_read!(items, expected, decoder, 4, 2, BufferTooSmall);
        framed_read!(items, expected, decoder, 4, 4, BufferTooSmall);

        // cspell: disable

        let expected: &[&[u8]] = &[b"jh asjd"];
        framed_read!(items, expected, decoder, 14, BufferTooSmall);

        let expected: &[&[u8]] = &[
            b"jh asjd",
            b"ppppppppppppppp",
            b"k hb",
            b"jsjuwjal kadj",
            b"jsadhjiu",
            b"w",
            b"jal kadjjsadhjiuwqens ",
            b"nd yxxcjajsdi",
            b"askdn asjdasd",
            b"iouqw essd",
        ];

        // cspell: enable

        framed_read!(items, expected, decoder, 32, BytesRemainingOnStream);
        framed_read!(items, expected, decoder, 32, 1, BytesRemainingOnStream);
        framed_read!(items, expected, decoder, 32, 2, BytesRemainingOnStream);
        framed_read!(items, expected, decoder, 32, 4, BytesRemainingOnStream);

        framed_read!(items, expected, decoder);
    }

    #[tokio::test]
    async fn sink_stream() {
        init_tracing();

        let items: Vec<Vec<u8>> = std::vec![
            b"Hello".to_vec(),
            b"Hello, world!".to_vec(),
            b"Hei".to_vec(),
            b"sup".to_vec(),
            b"Hey".to_vec(),
            b"He".to_vec(),
            b"H".to_vec(),
            b"".to_vec(),
        ];

        let decoder = Delimiter::new(b'#');
        let encoder = Delimiter::new(b'#');
        let map = |item: &[u8]| item.to_vec();

        sink_stream!(encoder, decoder, items, map);
    }
}
