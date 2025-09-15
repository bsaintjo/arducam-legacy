#![no_std]
#![no_main]

use arducam_legacy::{Arducam, Resolution};
use defmt::*;
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts,
    gpio::{Level, Output, Speed},
    i2c::{self, I2c},
    peripherals,
    spi::Spi,
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embassy_time::Timer;
use serde::Serialize;
use serde_json_core::{heapless::String, to_string};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(
    struct Irqs {
        I2C1 => i2c::EventInterruptHandler<peripherals::I2C1>, i2c::ErrorInterruptHandler<peripherals::I2C1>;
    }
);

#[derive(Serialize)]
struct Image<'a> {
    fifo_length: u32,
    bytes: &'a [u8],
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    let i2c_tx_dma = p.DMA1_CH3;
    let i2c_rx_dma = p.DMA1_CH4;
    let scl = p.PB8;
    let sda = p.PB9;
    let i2c = I2c::new(
        p.I2C1,
        scl,
        sda,
        Irqs,
        i2c_tx_dma,
        i2c_rx_dma,
        Default::default(),
    );

    let sck = p.PA5;
    let miso = p.PA6;
    let mosi = p.PA7;
    let spi_tx_dma = p.DMA1_CH1;
    let spi_rx_dma = p.DMA1_CH2;
    let spi = Spi::new(
        p.SPI1,
        sck,
        mosi,
        miso,
        spi_tx_dma,
        spi_rx_dma,
        Default::default(),
    );
    let bus: Mutex<CriticalSectionRawMutex, _> = Mutex::new(spi);
    let cs = Output::new(p.PB6, Level::High, Speed::VeryHigh);

    let device = SpiDevice::new(&bus, cs);

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
            Timer::after_millis(1000).await;
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
