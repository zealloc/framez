//! An example of using framez with embassy and esp-hal-embassy to do async UART I/O.
//!
//! This example will echo back any lines it receives over UART, prefixed with "Got line:".
//!
//! This example uses the lines codec so please ensure your terminal is set to send newline characters.
//!
//! You can interact with this example using https://github.com/hacknus/serial-monitor-rust

#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use embassy_executor::Spawner;
use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::timer::systimer::SystemTimer;
use esp_hal::uart::{Config, Uart};
use framez::codec::lines::StrLines;
use framez::{FramedRead, FramedWrite, next};

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    let (rx, tx) = Uart::new(peripherals.UART0, Config::default())
        .unwrap()
        .with_tx(peripherals.GPIO21)
        .with_rx(peripherals.GPIO20)
        .into_async()
        .split();

    let read_buf = &mut [0u8; 1024];
    let write_buf = &mut [0u8; 1024];

    let mut read = FramedRead::new(StrLines::new(), rx, read_buf);
    let mut write = FramedWrite::new(StrLines::new(), tx, write_buf);

    write
        .send("Hello from framez-async-uart-example!")
        .await
        .expect("UART write error");

    while let Some(line) = next!(read).transpose().expect("UART read error") {
        write.send("Got line:").await.expect("UART write error");
        write.send(line).await.expect("UART write error");
    }
}
