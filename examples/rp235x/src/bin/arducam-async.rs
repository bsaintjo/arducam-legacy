#![no_std]
#![no_main]

use arducam_legacy::{Arducam, Resolution};
use defmt::*;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_rp::{
    bind_interrupts, gpio::{Level, Output}, i2c::{I2c, InterruptHandler}, peripherals::I2C0, spi::{Config, Spi}
};
use embassy_sync::{
    blocking_mutex::raw::{CriticalSectionRawMutex, NoopRawMutex},
    mutex::Mutex,
};
use embassy_time::{Delay, Timer};
use embedded_hal_bus::spi::ExclusiveDevice;
use serde::Serialize;
use serde_json_core::{heapless::String, to_string};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    I2C0_IRQ => InterruptHandler<I2C0>;
});

#[derive(Serialize)]
struct Image<'a> {
    fifo_length: u32,
    bytes: &'a [u8],
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let scl = p.PIN_17;
    let sda = p.PIN_16;
    let i2c = I2c::new_async(
        p.I2C0,
        scl,
        sda,
        Irqs,
        Default::default(),
    );

    let sck = p.PIN_18;
    let miso = p.PIN_20;
    let mosi = p.PIN_19;
    let spi_tx_dma = p.DMA_CH0;
    let spi_rx_dma = p.DMA_CH1;
    let mut spi = Spi::new(
        p.SPI0,
        sck,
        mosi,
        miso,
        spi_tx_dma,
        spi_rx_dma,
        Config::default(),
    );
    let bus: Mutex<CriticalSectionRawMutex, _> = Mutex::new(spi);
    let cs = Output::new(p.PIN_21, Level::High);

    let device = SpiDevice::new(&bus, cs);
    let mut delay = Delay;

    let mut arducam = Arducam::new(i2c, device, Resolution::Res160x120);
    arducam.init().await.unwrap();

    let data = arducam.get_sensor_chipid().await.unwrap();
    info!("Whoami: 0x{:x}{:x}", data[0], data[1]);

    if arducam.is_connected().await.unwrap() {
        info!("Connected!");
    } else {
        info!("Disconnected?");
    }

    loop {
        arducam.start_capture().await.unwrap();
        info!("Capture started.");
        while !arducam.is_capture_done().await.unwrap() {
            info!("Capture in progress...");
            Timer::after_millis(50).await;
        }

        info!("Capture complete");
        let length = arducam.get_fifo_length().await.unwrap();
        info!("FIFO length: {}", length);
        let mut image = [0u8; 4096];
        arducam.read_captured_image(&mut image).await.unwrap();
        info!("Image read!");
        info!("First bytes {:02x}", image[..24]);
        info!("end? bytes {:02x}", image[3079..3120]);
        info!("Last bytes {:02x}", image[image.len() - 20..]);

        let image = Image {
            fifo_length: length,
            bytes: &image,
        };
        let as_string: Result<String<16000>, _> = to_string(&image);
        match as_string {
            Ok(s) => println!("{}", s.as_str()),
            Err(_) => println!("Buffer full"),
        }
    }
}
