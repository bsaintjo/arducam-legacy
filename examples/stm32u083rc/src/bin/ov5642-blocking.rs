#![no_std]
#![no_main]

use arducam_legacy::{ov5642::{self, Arducam5MP, Arducam5MPConfig}, Resolution};
use defmt::*;
use embassy_executor::Spawner;
use embassy_stm32::{
    gpio::{Level, Output, Speed},
    i2c::{Config, I2c},
    spi::Spi, time::Hertz,
};
use embassy_time::{Delay, Timer};
use embedded_hal_bus::spi::ExclusiveDevice;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());
    let scl = p.PB8;
    let sda = p.PB9;
    let mut i2c_config: Config = Default::default();
    i2c_config.frequency = Hertz(100 * 1000);
    i2c_config.gpio_speed = Speed::VeryHigh;
    let i2c = I2c::new_blocking(p.I2C1, scl, sda, i2c_config);

    let sck = p.PA5;
    let miso = p.PA6;
    let mosi = p.PA7;
    let spi = Spi::new_blocking(p.SPI1, sck, mosi, miso, Default::default());
    let cs = Output::new(p.PB6, Level::High, Speed::VeryHigh);

    let mut delay = Delay;
    let device = ExclusiveDevice::new_no_delay(spi, cs).unwrap();

    let config = Arducam5MPConfig {
        mode: ov5642::CameraMode::JPEG,
    };
    let mut arducam = Arducam5MP::new(i2c, device, config);
    let mut chip_id = 0u16;
    arducam.chip_id(&mut chip_id).unwrap();
    info!("Arducam chip ID: {:x}", chip_id);

    if arducam.spi_test(&mut delay).unwrap() {
        info!("SPI Test Successful");
    } else {
        error!("SPI Test Failed");
    }

    arducam.init(&mut delay).unwrap();
    info!("Initialized!");

    arducam.vsync_mask(&mut delay).unwrap();
    arducam.set_jpeg_size(Resolution::Res320x240).unwrap();
    Timer::after_millis(1000).await;
    arducam.clear_fifo_flag(&mut delay).unwrap();
    arducam.frames(&mut delay).unwrap();

    arducam.flush_fifo(&mut delay).unwrap();
    arducam.clear_fifo_flag(&mut delay).unwrap();
    arducam.start_capture(&mut delay).unwrap();
    info!("Capture started!");

    // Timer::after_millis(200).await;

    while arducam.is_capture_done().unwrap() {
        Timer::after_millis(200).await;
    }
    info!("Capture complete!");

    let mut fifo_length = 0u32;
    arducam.fifo_length(&mut fifo_length).unwrap();
    info!("FIFO Length: {}", fifo_length);

    let mut image = [0u8; 10000];
    arducam.read_captured_image(&mut image).unwrap();
    info!("Image read!");
    info!("First bytes {:02x}", image[..50]);

    loop {}
}
