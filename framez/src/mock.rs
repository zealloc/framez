//! Noop implementation of embedded-io-async traits for testing purposes.

use core::convert::Infallible;

use embedded_io_async::{ErrorType, Read, Write};

#[derive(Debug)]
pub struct Noop;

impl ErrorType for Noop {
    type Error = Infallible;
}

impl Read for Noop {
    async fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        Ok(buf.len())
    }
}

impl Write for Noop {
    async fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
        Ok(buf.len())
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}
