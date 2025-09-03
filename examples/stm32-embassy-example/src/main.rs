#![no_std]
#![no_main]

use arducam_legacy::{Arducam, Resolution};
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{i2c::I2c, spi::Spi};
use embassy_time::Delay;
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
    // let cs = p.PB6;
    let spi = Spi::new_blocking(p.SPI1, sck, mosi, miso, Default::default());

    let mut delay = Delay;
    let mut arducam = Arducam::new_blocking(i2c, spi, Resolution::Res160x120);
    arducam.init(&mut delay).unwrap();

    let data = arducam.get_sensor_chipid().unwrap();
    info!("Whoami: 0x{:x}{:x}", data[0], data[1]);

    if arducam.is_connected().unwrap() {
        info!("Connected!");
    } else {
        info!("Disconnected?");
    }
}
