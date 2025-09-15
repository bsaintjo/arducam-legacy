#![no_std]
#![no_main]

use arducam_legacy::{
    ov5642::{self, Arducam5MP, Arducam5MPConfig},
    Resolution,
};
use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::{
    bind_interrupts,
    gpio::{Level, Output},
    i2c::I2c,
    peripherals::USB,
    spi::Spi,
    usb::{Driver, InterruptHandler},
};
use embassy_time::{Delay, Timer};
use embassy_usb::{class::cdc_acm::{CdcAcmClass, State}, UsbDevice};
use embedded_hal_bus::spi::ExclusiveDevice;
use static_cell::StaticCell;
// use serde::Serialize;
// use serde_json_core::{heapless::String, to_string};
use {defmt_rtt as _, panic_probe as _};

// #[derive(Serialize)]
// struct Image<'a> {
//     fifo_length: u32,
//     bytes: &'a [u8]
// }

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

type UDevice = UsbDevice<'static, Driver<'static, USB>>;

#[embassy_executor::task]
async fn usb_task(mut usb: UDevice) -> ! {
    usb.run().await
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let driver = Driver::new(p.USB, Irqs);

    let config = {
        let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
        config.manufacturer = Some("Embassy");
        config.product = Some("USB-serial example");
        config.serial_number = Some("12345678");
        config.max_power = 100;
        config.max_packet_size_0 = 64;
        config
    };

    let mut builder = {
        static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
        static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
        static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();

        let builder = embassy_usb::Builder::new(
            driver,
            config,
            CONFIG_DESCRIPTOR.init([0; 256]),
            BOS_DESCRIPTOR.init([0; 256]),
            &mut [], // no msos descriptors
            CONTROL_BUF.init([0; 64]),
        );
        builder
    };

    let mut class = {
        static STATE: StaticCell<State> = StaticCell::new();
        let state = STATE.init(State::new());
        CdcAcmClass::new(&mut builder, state, 64)
    };

    let usb = builder.build();
    spawner.spawn(usb_task(usb));
    class.wait_connection().await;

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
    arducam
        .set_jpeg_size(Resolution::Res320x240, &mut delay)
        .unwrap();
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
            .read_captured_image(&mut image[..fifo_length as usize])
            .unwrap();
        info!("Image read!");
        info!("First bytes {:02x}", image[..50]);
        info!(
            "Last bytes {:02x}",
            image[fifo_length as usize - 20..fifo_length as usize]
        );
        info!(
            "Start marker: {:?}",
            image.windows(2).position(|pair| pair == [0xff, 0xd8])
        );
        info!(
            "End marker: {:?}",
            image.windows(2).position(|pair| pair == [0xff, 0xd9])
        );
        class.write_packet(&image[..fifo_length as usize]).await;
        arducam.clear_fifo_flag(&mut delay).unwrap();
    }
}
