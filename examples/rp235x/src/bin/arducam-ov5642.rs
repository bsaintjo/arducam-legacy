#![no_std]
#![no_main]

use arducam_legacy::{
    ov5642::{self, Arducam5MP, Arducam5MPConfig},
    Resolution,
};
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::{
    gpio::{Level, Output},
    i2c::I2c,
    spi::Spi,
};
use embassy_time::{Delay, Timer};
use embedded_hal_bus::spi::ExclusiveDevice;
// use serde::Serialize;
// use serde_json_core::{heapless::String, to_string};
use {defmt_rtt as _, panic_probe as _};

// #[derive(Serialize)]
// struct Image<'a> {
//     fifo_length: u32,
//     bytes: &'a [u8]
// }

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let scl = p.PIN_17;
    let sda = p.PIN_16;
    let i2c = I2c::new_blocking(p.I2C0, scl, sda, Default::default());

    let sck = p.PIN_18;
    let miso = p.PIN_20;
    let mosi = p.PIN_19;
    let spi = Spi::new_blocking(p.SPI0, sck, mosi, miso, Default::default());
    let cs = Output::new(p.PIN_21, Level::High);

    let mut delay = Delay;
    let device = ExclusiveDevice::new_no_delay(spi, cs).unwrap();

    let arducam_config = Arducam5MPConfig {
        mode: ov5642::CameraMode::JPEG,
    };
    let mut arducam = Arducam5MP::new(i2c, device, arducam_config);
    arducam.init(&mut delay).unwrap();

    let mut chip_id = 0u16;
    arducam.chip_id(&mut chip_id).unwrap();
    info!("Whoami: 0x{:x}", chip_id);

    if arducam.spi_test(&mut delay).unwrap() {
        info!("SPI TEST Succesfull!");
    } else {
        info!("SPI TEST FAILED?");
    }
    arducam.init(&mut delay).unwrap();
    info!("Initialized!");

    arducam.vsync_mask(&mut delay).unwrap();
    arducam.set_jpeg_size(Resolution::Res320x240, &mut delay).unwrap();
    // Timer::after_millis(1000).await;
    arducam.clear_fifo_flag(&mut delay).unwrap();
    arducam.frames(&mut delay).unwrap();

    let mut image = [0u8; 200000];
    loop {
        arducam.flush_fifo(&mut delay).unwrap();
        arducam.clear_fifo_flag(&mut delay).unwrap();
        arducam.start_capture(&mut delay).unwrap();
        info!("Capture started!");

        // Timer::after_millis(200).await;

        while !arducam.is_capture_done().unwrap() {
            Timer::after_millis(100).await;
        }
        info!("Capture complete!");

        let mut fifo_length = 0u32;
        arducam.fifo_length(&mut fifo_length).unwrap();
        info!("FIFO Length: {}", fifo_length);

        arducam
            .read_captured_image(&mut image[..fifo_length as usize + 1])
            .unwrap();
        info!("Image read!");
        info!("First bytes {:02x}", image[..50]);
        info!(
            "Last bytes {:02x}",
            image[fifo_length as usize - 50..fifo_length as usize + 10]
        );
        info!(
            "Start marker: {:?}",
            image.windows(2).position(|pair| pair == [0xff, 0xd8])
        );
        info!(
            "End marker: {:?}",
            image.windows(2).position(|pair| pair == [0xff, 0xd9])
        );
        arducam.clear_fifo_flag(&mut delay);
    }
}
