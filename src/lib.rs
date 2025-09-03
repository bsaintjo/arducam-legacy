//! This library aims to provide support for older legacy Arducam cameras such as ArduCAM Mini 2MP Plus
//! It provides `embedded-hal` compatible API

#![no_std]
#![no_main]

use core::{slice::IterMut};

use embedded_hal_1::{delay::DelayNs, i2c::I2c, spi::SpiBus};
use registers::*;

pub mod registers;

const ARDUCHIP_TEST1: u8 = 0x00;
const ARDUCHIP_FIFO: u8 = 0x04;
const ARDUCHIP_TRIG: u8 = 0x41;
const OV2640_ADDR: u8 = 0x30;
const OV2640_CHIPID_HIGH: u8 = 0x0A;
const OV2640_CHIPID_LOW: u8 = 0x0B;
const FIFO_CLEAR_MASK: u8 = 0x01;
const FIFO_START_MASK: u8 = 0x02;
const FIFO_BURST: u8 = 0x3C;
const FIFO_SIZE1: u8 = 0x42;
const FIFO_SIZE2: u8 = 0x43;
const FIFO_SIZE3: u8 = 0x44;
const CAP_DONE_MASK: u8 = 0x08;

#[derive(Debug)]
// Image resolutions
pub enum Resolution {
    Res160x120,
    Res176x144,
    Res320x240,
    Res352x288,
    Res640x480,
    Res800x600,
    Res1024x768,
    Res1280x1024,
    Res1600x1200,
}

#[derive(PartialEq, Eq, Debug)]
/// Image formats which Arducam can handle
pub enum ImageFormat {
    // BMP,
    // RAW,
    JPEG,
}

pub struct Arducam<I: I2c, S> {
    pub i2c: I,
    pub spi: S,
    resolution: Resolution,
}

#[derive(Debug)]
pub enum ArduCAMError {
    I2cError,
    SpiError,
}

impl<I: I2c, S: embedded_hal_async::spi::SpiBus> Arducam<I, S> {
    pub fn new(i2c: I, spi: S, resolution: Resolution) -> Self {
        Self {
            i2c,
            spi,
            resolution,
        }
    }
}

impl<I: I2c, S: SpiBus> Arducam<I, S> {
    pub fn new_blocking(i2c: I, spi: S, resolution: Resolution) -> Self {
        Self {
            i2c,
            spi,
            resolution,
        }
    }

    fn sensor_readreg8_8(&mut self, reg: u8, out: &mut [u8]) -> Result<(), ArduCAMError> {
        self.i2c
            .write_read(OV2640_ADDR, &[reg & 0xFF], out)
            .map_err(|_| ArduCAMError::I2cError)
    }

    fn sensor_writereg8_8(&mut self, reg: u8, data: u8) -> Result<(), ArduCAMError> {
        self.i2c
            .write(OV2640_ADDR, &[reg & 0xFF, data & 0xFF])
            .map_err(|_| ArduCAMError::I2cError)
    }

    fn arduchip_write(&mut self, addr: u8, data: u8) -> Result<(), ArduCAMError> {
        // self.spi_cs.set_low().map_err(Error::Pin)?;
        self.spi
            .write(&[addr; 1])
            .map_err(|_| ArduCAMError::SpiError)?;
        self.spi
            .write(&[data; 1])
            .map_err(|_| ArduCAMError::SpiError)?;
        // self.spi_cs.set_high().map_err(Error::Pin)?;
        Ok(())
    }

    fn arduchip_read(&mut self, addr: u8) -> Result<u8, ArduCAMError> {
        // self.spi_cs.set_low().map_err(Error::Pin)?;
        self.spi
            .write(&mut [addr; 1])
            .map_err(|_| ArduCAMError::SpiError)?;
        let mut value = [0u8; 1];
        self.spi
            .read(&mut value)
            .map_err(|_| ArduCAMError::SpiError)?;
        // self.spi_cs.set_high().map_err(Error::Pin)?;
        Ok(value[0])
    }

    fn arduchip_write_reg(&mut self, addr: u8, data: u8) -> Result<(), ArduCAMError> {
        self.arduchip_write(addr | 0x80, data)
    }

    fn arduchip_read_reg(&mut self, addr: u8) -> Result<u8, ArduCAMError> {
        self.arduchip_read(addr & 0x7F)
    }

    fn sensor_writeregs8_8(&mut self, regs: &[[u8; 2]]) -> Result<(), ArduCAMError> {
        for reg in regs {
            self.sensor_writereg8_8(reg[0], reg[1])?;
        }
        Ok(())
    }

    fn send_resolution(&mut self) -> Result<(), ArduCAMError> {
        match self.resolution {
            Resolution::Res160x120 => self.sensor_writeregs8_8(&OV2640_160x120_JPEG),
            Resolution::Res1024x768 => self.sensor_writeregs8_8(&OV2640_1024x768_JPEG),
            Resolution::Res1280x1024 => self.sensor_writeregs8_8(&OV2640_1280x1024_JPEG),
            Resolution::Res1600x1200 => self.sensor_writeregs8_8(&OV2640_1600x1200_JPEG),
            Resolution::Res176x144 => self.sensor_writeregs8_8(&OV2640_176x144_JPEG),
            Resolution::Res320x240 => self.sensor_writeregs8_8(&OV2640_320x240_JPEG),
            Resolution::Res352x288 => self.sensor_writeregs8_8(&OV2640_352x288_JPEG),
            Resolution::Res640x480 => self.sensor_writeregs8_8(&OV2640_640x480_JPEG),
            Resolution::Res800x600 => self.sensor_writeregs8_8(&OV2640_800x600_JPEG),
        }
    }

    fn flush_fifo(&mut self) -> Result<(), ArduCAMError> {
        self.arduchip_write_reg(ARDUCHIP_FIFO, FIFO_CLEAR_MASK)
    }

    fn start_fifo(&mut self) -> Result<(), ArduCAMError> {
        self.arduchip_write_reg(ARDUCHIP_FIFO, FIFO_START_MASK)
    }

    fn set_fifo_burst(&mut self) -> Result<(), ArduCAMError> {
        self.spi
            .write(&[FIFO_BURST])
            .map_err(|_| ArduCAMError::SpiError)
    }

    /// Initializes Arducam to resetted state
    pub fn init<D>(&mut self, delay: &mut D) -> Result<(), ArduCAMError>
    where
        D: DelayNs,
    {
        self.arduchip_write_reg(0x07, 0x80)?;
        delay.delay_ms(100);
        self.arduchip_write_reg(0x07, 0x00)?;
        delay.delay_ms(100);
        self.sensor_writereg8_8(0xFF, 0x01)?;
        self.sensor_writereg8_8(0x12, 0x80)?;
        delay.delay_ms(100);

        self.sensor_writeregs8_8(&OV2640_JPEG_INIT)?;
        self.sensor_writeregs8_8(&OV2640_YUV422)?;
        self.sensor_writereg8_8(0xFF, 0x01)?;
        self.sensor_writereg8_8(0x15, 0x00)?;
        self.send_resolution()?;

        Ok(())
    }

    /// Sets camera resolution
    pub fn set_resolution(&mut self, resolution: Resolution) -> Result<(), ArduCAMError> {
        self.resolution = resolution;
        self.send_resolution()?;
        Ok(())
    }

    /// Checks if Arducam is still connected to SPI bus
    pub fn is_connected(&mut self) -> Result<bool, ArduCAMError> {
        let test_value = 0x52;
        self.arduchip_write_reg(ARDUCHIP_TEST1, test_value)?;
        let result = self.arduchip_read_reg(ARDUCHIP_TEST1)?;

        let valid_ov2640_chipid1 = [0x26, 0x41];
        let valid_ov2640_chipid2 = [0x26, 0x42];
        let chipid = self.get_sensor_chipid()?;
        defmt::info!("result {:x}, expected {:x}", result, test_value);
        defmt::info!("chipid {:x}{:x}", chipid[0], chipid[1]);

        if test_value == result && chipid == valid_ov2640_chipid1 || chipid == valid_ov2640_chipid2
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Sends image capture request
    pub fn start_capture(&mut self) -> Result<(), ArduCAMError> {
        self.flush_fifo()?;
        self.start_fifo()?;
        Ok(())
    }

    /// Checks if image capture is done
    pub fn is_capture_done(&mut self) -> Result<bool, ArduCAMError> {
        self.arduchip_read_reg(ARDUCHIP_TRIG)
            .map(|result| result & CAP_DONE_MASK != 0)
    }

    /// Saves captured image to provided mutable slice
    /// It is important to be sure if that slice will be big enough for image data
    /// otherwise data will be cut
    ///
    /// # Returns
    /// Actual image size
    pub fn read_captured_image(&mut self, data_out: IterMut<u8>) -> Result<usize, ArduCAMError> {
        let length = self.get_fifo_length()?;
        let mut final_length = 0;
        // self.spi_cs.set_low().map_err(Error::Pin)?;
        self.set_fifo_burst()?;
        let mut curr_byte = 0;
        #[allow(unused_assignments)]
        let mut prev_byte = 0;
        for (i, b) in data_out.enumerate() {
            prev_byte = curr_byte;
            let mut curr_byte_tmp = [0u8; 1];
            self.spi
                .read(&mut curr_byte_tmp)
                .map_err(|_| ArduCAMError::SpiError)?;
            curr_byte = curr_byte_tmp[0];
            *b = curr_byte;
            if prev_byte == 0xFF && curr_byte == 0xD9 || i as u32 > length {
                final_length = i;
                break;
            }
        }
        self.flush_fifo()?;
        Ok(final_length)
    }

    /// Returns image length reported by arduchip in FIFO
    pub fn get_fifo_length(&mut self) -> Result<u32, ArduCAMError> {
        let mut len_builder = (0u32, 0u32, 0u32);
        len_builder.0 = self.arduchip_read_reg(FIFO_SIZE1)?.into();
        len_builder.1 = self.arduchip_read_reg(FIFO_SIZE2)?.into();
        len_builder.2 = (self.arduchip_read_reg(FIFO_SIZE3)? & 0x7F).into();
        Ok((len_builder.2 << 16 | len_builder.1 << 8 | len_builder.0) as u32 & 0x7FFFFFu32)
    }

    /// Returns sensor vendor and product ID
    pub fn get_sensor_chipid(&mut self) -> Result<[u8; 2], ArduCAMError> {
        let mut chipid: [u8; 2] = [0; 2];
        self.sensor_writereg8_8(0xFF, 0x01)?;
        self.sensor_readreg8_8(OV2640_CHIPID_HIGH, &mut chipid[0..1])?;
        self.sensor_readreg8_8(OV2640_CHIPID_LOW, &mut chipid[1..2])?;
        Ok(chipid)
    }
}
