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
use serde::Serialize;
use serde_json_core::{heapless::String, to_string};
use {defmt_rtt as _, panic_probe as _};

#[derive(Serialize)]
struct Image<'a> {
    fifo_length: u32,
    bytes: &'a [u8]
}

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

    loop {
        arducam.start_capture().unwrap();
        info!("Capture started.");
        while !arducam.is_capture_done().unwrap() {
            // info!("Capture in progress...");
            Timer::after_millis(50).await;
        }

        info!("Capture complete");
        let length = arducam.get_fifo_length().unwrap();
        info!("FIFO length: {}", length);
        let mut image = [0u8; 4096];
        arducam.read_captured_image(&mut image).unwrap();
        info!("Image read!");
        info!("First bytes {:02x}", image[..24]);
        info!("end? bytes {:02x}", image[3079..3120]);
        info!("Last bytes {:02x}", image[image.len() - 20..]);

        let image = Image { fifo_length: length, bytes: &image };
        let as_string: Result<String<16000>, _> = to_string(&image);
        match as_string {
            Ok(s) => println!("{}", s.as_str()),
            Err(_) => println!("Buffer full"),
        }
    }
}
