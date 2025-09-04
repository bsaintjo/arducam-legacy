#![no_std]
#![no_main]

use arducam_legacy::{Arducam, Resolution};
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{
    gpio::{Level, Output, Speed},
    i2c::I2c,
    spi::Spi,
};
use embassy_time::{Delay, Timer};
use embedded_hal_bus::spi::ExclusiveDevice;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    let scl = p.PB8;
    let sda = p.PB9;
    let i2c = I2c::new_blocking(p.I2C1, scl, sda, Default::default());

    let sck = p.PA5;
    let miso = p.PA6;
    let mosi = p.PA7;
    let spi = Spi::new_blocking(p.SPI1, sck, mosi, miso, Default::default());
    let cs = Output::new(p.PB6, Level::High, Speed::VeryHigh);

    let mut delay = Delay;
    let device = ExclusiveDevice::new(spi, cs, &mut delay).unwrap();

    let mut arducam = Arducam::new_blocking(i2c, device, Resolution::Res160x120);
    arducam.init().unwrap();

    let data = arducam.get_sensor_chipid().unwrap();
    info!("Whoami: 0x{:x}{:x}", data[0], data[1]);

    if arducam.is_connected().unwrap() {
        info!("Connected!");
    } else {
        info!("Disconnected?");
    }

    arducam.start_capture().unwrap();
    info!("Capture started.");
    while !arducam.is_capture_done().unwrap() {
        // info!("Capture in progress...");
        Timer::after_millis(50).await;
    }

    info!("Capture complete");
    let length = arducam.get_fifo_length().unwrap();
    info!("FIFO length: {}", length);
    let mut image = [0u8; 8192];
    arducam.read_captured_image(&mut image).unwrap();
    info!("Image read!");
    info!("First bytes {:02x}", image[..12]);
}
