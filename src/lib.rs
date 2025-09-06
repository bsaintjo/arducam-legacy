//! This library aims to provide support for older legacy Arducam cameras such as ArduCAM Mini 2MP Plus
//! It provides `embedded-hal` compatible API

#![no_std]
#![no_main]

#![allow(async_fn_in_trait)]

use core::marker::PhantomData;

use embedded_hal_1::{
    i2c::{self, Error as I2cError, I2c},
    spi::{self, Error as SpiError, Operation, SpiDevice},
};
use embedded_hal_async::{i2c::I2c as AsyncI2c, spi::SpiDevice as AsyncSpiDevice};
use ov2640_registers::*;

pub mod ov2640_registers;
pub mod ov5642_registers;

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

pub struct Blocking;
pub struct Async;

pub struct Arducam<M, I, S> {
    mode: PhantomData<M>,
    pub i2c: I,
    pub spi: S,
    resolution: Resolution,
}

#[derive(thiserror::Error, Debug)]
pub enum ArducamError {
    #[error("I2C error: {0}")]
    I2cError(i2c::ErrorKind),
    #[error("SPI error: {0}")]
    SpiError(spi::ErrorKind),
}

impl<I: AsyncI2c, S: AsyncSpiDevice> Arducam<Async, I, S> {
    pub fn new(i2c: I, spi: S, resolution: Resolution) -> Self {
        Self {
            mode: PhantomData,
            i2c,
            spi,
            resolution,
        }
    }

    async fn sensor_readreg8_8(&mut self, reg: u8, out: &mut [u8]) -> Result<(), ArducamError> {
        self.i2c
            .write_read(OV2640_ADDR, &[reg], out)
            .await
            .map_err(|e| ArducamError::I2cError(e.kind()))
    }

    async fn sensor_writereg8_8(&mut self, reg: u8, data: u8) -> Result<(), ArducamError> {
        self.i2c
            .write(OV2640_ADDR, &[reg, data])
            .await
            .map_err(|e| ArducamError::I2cError(e.kind()))
    }

    async fn arduchip_write_reg(&mut self, addr: u8, data: u8) -> Result<(), ArducamError> {
        // self.arduchip_write(addr | 0x80, data)
        // self.spi
        //     .write(&[addr | 0x80, data])
        //     .await
        //     .map_err(|e| ArducamError::SpiError(e.kind()))?;
        // Ok(())
        self.spi
            .transaction(&mut [Operation::Write(&[addr | 0x80]), Operation::Write(&[data])])
            .await
            .map_err(|e| ArducamError::SpiError(e.kind()))?;
        Ok(())
    }

    async fn transaction(
        &mut self,
        operations: &mut [Operation<'_, u8>],
    ) -> Result<(), ArducamError> {
        self.spi
            .transaction(operations)
            .await
            .map_err(|e| ArducamError::SpiError(e.kind()))
    }

    async fn arduchip_read_reg(&mut self, addr: u8) -> Result<u8, ArducamError> {
        // self.arduchip_read(addr & 0x7F)
        let mut value = [0u8; 1];
        self.transaction(&mut [
            Operation::Write(&[addr & 0x7f]),
            Operation::Read(&mut value),
        ])
        .await?;
        Ok(value[0])
    }

    async fn sensor_writeregs8_8(&mut self, regs: &[[u8; 2]]) -> Result<(), ArducamError> {
        for reg in regs {
            self.sensor_writereg8_8(reg[0], reg[1]).await?;
        }
        Ok(())
    }

    async fn send_resolution(&mut self) -> Result<(), ArducamError> {
        match self.resolution {
            Resolution::Res160x120 => self.sensor_writeregs8_8(&OV2640_160x120_JPEG).await,
            Resolution::Res1024x768 => self.sensor_writeregs8_8(&OV2640_1024x768_JPEG).await,
            Resolution::Res1280x1024 => self.sensor_writeregs8_8(&OV2640_1280x1024_JPEG).await,
            Resolution::Res1600x1200 => self.sensor_writeregs8_8(&OV2640_1600x1200_JPEG).await,
            Resolution::Res176x144 => self.sensor_writeregs8_8(&OV2640_176x144_JPEG).await,
            Resolution::Res320x240 => self.sensor_writeregs8_8(&OV2640_320x240_JPEG).await,
            Resolution::Res352x288 => self.sensor_writeregs8_8(&OV2640_352x288_JPEG).await,
            Resolution::Res640x480 => self.sensor_writeregs8_8(&OV2640_640x480_JPEG).await,
            Resolution::Res800x600 => self.sensor_writeregs8_8(&OV2640_800x600_JPEG).await,
        }
    }

    async fn flush_fifo(&mut self) -> Result<(), ArducamError> {
        self.arduchip_write_reg(ARDUCHIP_FIFO, FIFO_CLEAR_MASK)
            .await
    }

    async fn start_fifo(&mut self) -> Result<(), ArducamError> {
        self.arduchip_write_reg(ARDUCHIP_FIFO, FIFO_START_MASK)
            .await
    }

    pub async fn init(&mut self) -> Result<(), ArducamError> {
        self.arduchip_write_reg(0x07, 0x80).await?;
        self.transaction(&mut [Operation::DelayNs(100_000_000)])
            .await?;
        self.arduchip_write_reg(0x07, 0x00).await?;
        self.transaction(&mut [Operation::DelayNs(100_000_000)])
            .await?;

        self.sensor_writereg8_8(0xFF, 0x01).await?;
        self.transaction(&mut [Operation::DelayNs(100_000_000)])
            .await?;

        self.sensor_writereg8_8(0x12, 0x80).await?;
        self.transaction(&mut [Operation::DelayNs(100_000_000)])
            .await?;

        self.sensor_writeregs8_8(&OV2640_JPEG_INIT).await?;
        self.sensor_writeregs8_8(&OV2640_YUV422).await?;
        self.sensor_writeregs8_8(&OV2640_JPEG).await?;
        self.sensor_writereg8_8(0xFF, 0x01).await?;
        self.sensor_writereg8_8(0x15, 0x00).await?;
        self.send_resolution().await?;

        Ok(())
    }

    /// Sets camera resolution
    pub async fn set_resolution(&mut self, resolution: Resolution) -> Result<(), ArducamError> {
        self.resolution = resolution;
        self.send_resolution().await?;
        Ok(())
    }

    /// Checks if Arducam is still connected to SPI bus
    pub async fn is_connected(&mut self) -> Result<bool, ArducamError> {
        let test_value = 0x52;
        self.arduchip_write_reg(ARDUCHIP_TEST1, test_value).await?;
        let result = self.arduchip_read_reg(ARDUCHIP_TEST1).await?;

        let valid_ov2640_chipid1 = [0x26, 0x41];
        let valid_ov2640_chipid2 = [0x26, 0x42];
        let chipid = self.get_sensor_chipid().await?;
        defmt::debug!("result {:x}, expected {:x}", result, test_value);
        defmt::debug!("chipid {:x}{:x}", chipid[0], chipid[1]);

        if test_value == result && chipid == valid_ov2640_chipid1 || chipid == valid_ov2640_chipid2
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Sends image capture request
    pub async fn start_capture(&mut self) -> Result<(), ArducamError> {
        self.flush_fifo().await?;
        self.start_fifo().await?;
        Ok(())
    }

    /// Checks if image capture is done
    pub async fn is_capture_done(&mut self) -> Result<bool, ArducamError> {
        self.arduchip_read_reg(ARDUCHIP_TRIG)
            .await
            .map(|result| result & CAP_DONE_MASK != 0)
    }

    /// Saves captured image to provided mutable slice
    /// It is important to be sure if that slice will be big enough for image data
    /// otherwise data will be cut
    ///
    /// # Returns
    /// Actual image size
    pub async fn read_captured_image(&mut self, out: &mut [u8]) -> Result<(), ArducamError> {
        self.transaction(&mut [
            Operation::Write(&[FIFO_BURST & 0x7f]),
            Operation::Read(out),
            Operation::Write(&[ARDUCHIP_FIFO | 0x80, FIFO_CLEAR_MASK]),
        ])
        .await
    }

    /// Returns image length reported by arduchip in FIFO
    pub async fn get_fifo_length(&mut self) -> Result<u32, ArducamError> {
        let mut len_builder = (0u32, 0u32, 0u32);
        len_builder.0 = self.arduchip_read_reg(FIFO_SIZE1).await?.into();
        len_builder.1 = self.arduchip_read_reg(FIFO_SIZE2).await?.into();
        len_builder.2 = (self.arduchip_read_reg(FIFO_SIZE3).await? & 0x7F).into();
        Ok((len_builder.2 << 16 | len_builder.1 << 8 | len_builder.0) as u32 & 0x7FFFFFu32)
    }

    /// Returns sensor vendor and product ID
    pub async fn get_sensor_chipid(&mut self) -> Result<[u8; 2], ArducamError> {
        let mut chipid: [u8; 2] = [0; 2];
        self.sensor_writereg8_8(0xFF, 0x01).await?;
        self.sensor_readreg8_8(OV2640_CHIPID_HIGH, &mut chipid[0..1])
            .await?;
        self.sensor_readreg8_8(OV2640_CHIPID_LOW, &mut chipid[1..2])
            .await?;
        Ok(chipid)
    }
}

impl<I: I2c, S: SpiDevice> Arducam<Blocking, I, S> {
    pub fn new_blocking(i2c: I, spi: S, resolution: Resolution) -> Self {
        Self {
            mode: PhantomData,
            i2c,
            spi,
            resolution,
        }
    }

    fn sensor_readreg8_8(&mut self, reg: u8, out: &mut [u8]) -> Result<(), ArducamError> {
        self.i2c
            .write_read(OV2640_ADDR, &[reg], out)
            .map_err(|e| ArducamError::I2cError(e.kind()))
    }

    fn blocking_sensor_writereg8_8(&mut self, reg: u8, data: u8) -> Result<(), ArducamError> {
        self.i2c
            .write(OV2640_ADDR, &[reg, data])
            .map_err(|e| ArducamError::I2cError(e.kind()))
    }

    // fn arduchip_write(&mut self, addr: u8, data: u8) -> Result<(), ArducamError> {
    // self.spi
    //     .write(&[addr, data])
    //     .map_err(|e| ArducamError::SpiError(e.kind()))
    // }

    // fn arduchip_read(&mut self, addr: u8) -> Result<u8, ArducamError> {
    //     let mut value = [0u8; 1];
    //     self.transaction(&mut [Operation::Write(&[addr; 1]), Operation::Read(&mut value)])?;
    //     Ok(value[0])
    // }

    fn arduchip_write_reg(&mut self, addr: u8, data: u8) -> Result<(), ArducamError> {
        // self.arduchip_write(addr | 0x80, data)
        self.spi
            .write(&[addr | 0x80, data])
            .map_err(|e| ArducamError::SpiError(e.kind()))
    }

    fn arduchip_read_reg(&mut self, addr: u8) -> Result<u8, ArducamError> {
        // self.arduchip_read(addr & 0x7F)
        let mut value = [0u8; 1];
        self.transaction(&mut [
            Operation::Write(&[addr & 0x7f]),
            Operation::Read(&mut value),
        ])?;
        Ok(value[0])
    }

    fn sensor_writeregs8_8(&mut self, regs: &[[u8; 2]]) -> Result<(), ArducamError> {
        for reg in regs {
            self.blocking_sensor_writereg8_8(reg[0], reg[1])?;
        }
        Ok(())
    }

    fn send_resolution(&mut self) -> Result<(), ArducamError> {
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

    fn flush_fifo(&mut self) -> Result<(), ArducamError> {
        self.arduchip_write_reg(ARDUCHIP_FIFO, FIFO_CLEAR_MASK)
    }

    fn start_fifo(&mut self) -> Result<(), ArducamError> {
        self.arduchip_write_reg(ARDUCHIP_FIFO, FIFO_START_MASK)
    }

    // fn set_fifo_burst(&mut self) -> Result<(), ArduCAMError> {
    //     self.spi
    //         .write(&[FIFO_BURST])
    //         .map_err(|_| ArduCAMError::SpiError)
    // }
    fn transaction(&mut self, operations: &mut [Operation<'_, u8>]) -> Result<(), ArducamError> {
        self.spi
            .transaction(operations)
            .map_err(|e| ArducamError::SpiError(e.kind()))
    }

    /// Initializes Arducam to resetted state
    pub fn init(&mut self) -> Result<(), ArducamError> {
        self.arduchip_write_reg(0x07, 0x80)?;
        self.transaction(&mut [Operation::DelayNs(100_000_000)])?;
        self.arduchip_write_reg(0x07, 0x00)?;
        self.transaction(&mut [Operation::DelayNs(100_000_000)])?;

        self.blocking_sensor_writereg8_8(0xFF, 0x01)?;
        self.transaction(&mut [Operation::DelayNs(100_000_000)])?;

        self.blocking_sensor_writereg8_8(0x12, 0x80)?;
        self.transaction(&mut [Operation::DelayNs(100_000_000)])?;

        self.sensor_writeregs8_8(&OV2640_JPEG_INIT)?;
        self.sensor_writeregs8_8(&OV2640_YUV422)?;
        self.sensor_writeregs8_8(&OV2640_JPEG)?;
        self.blocking_sensor_writereg8_8(0xFF, 0x01)?;
        self.blocking_sensor_writereg8_8(0x15, 0x00)?;
        self.send_resolution()?;

        Ok(())
    }

    /// Sets camera resolution
    pub fn set_resolution(&mut self, resolution: Resolution) -> Result<(), ArducamError> {
        self.resolution = resolution;
        self.send_resolution()?;
        Ok(())
    }

    /// Checks if Arducam is still connected to SPI bus
    pub fn is_connected(&mut self) -> Result<bool, ArducamError> {
        let test_value = 0x52;
        self.arduchip_write_reg(ARDUCHIP_TEST1, test_value)?;
        let result = self.arduchip_read_reg(ARDUCHIP_TEST1)?;

        let valid_ov2640_chipid1 = [0x26, 0x41];
        let valid_ov2640_chipid2 = [0x26, 0x42];
        let chipid = self.get_sensor_chipid()?;
        defmt::debug!("result {:x}, expected {:x}", result, test_value);
        defmt::debug!("chipid {:x}{:x}", chipid[0], chipid[1]);

        if test_value == result && chipid == valid_ov2640_chipid1 || chipid == valid_ov2640_chipid2
        {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Sends image capture request
    pub fn start_capture(&mut self) -> Result<(), ArducamError> {
        self.flush_fifo()?;
        self.start_fifo()?;
        Ok(())
    }

    /// Checks if image capture is done
    pub fn is_capture_done(&mut self) -> Result<bool, ArducamError> {
        self.arduchip_read_reg(ARDUCHIP_TRIG)
            .map(|result| result & CAP_DONE_MASK != 0)
    }

    /// Saves captured image to provided mutable slice
    /// It is important to be sure if that slice will be big enough for image data
    /// otherwise data will be cut
    ///
    /// # Returns
    /// Actual image size
    pub fn read_captured_image(&mut self, out: &mut [u8]) -> Result<(), ArducamError> {
        self.transaction(&mut [
            Operation::Write(&[FIFO_BURST & 0x7f]),
            Operation::Read(out),
            Operation::Write(&[ARDUCHIP_FIFO | 0x80, FIFO_CLEAR_MASK]),
        ])
    }

    /// Returns image length reported by arduchip in FIFO
    pub fn get_fifo_length(&mut self) -> Result<u32, ArducamError> {
        let mut len_builder = (0u32, 0u32, 0u32);
        len_builder.0 = self.arduchip_read_reg(FIFO_SIZE1)?.into();
        len_builder.1 = self.arduchip_read_reg(FIFO_SIZE2)?.into();
        len_builder.2 = (self.arduchip_read_reg(FIFO_SIZE3)? & 0x7F).into();
        Ok((len_builder.2 << 16 | len_builder.1 << 8 | len_builder.0) as u32 & 0x7FFFFFu32)
    }

    /// Returns sensor vendor and product ID
    pub fn get_sensor_chipid(&mut self) -> Result<[u8; 2], ArducamError> {
        let mut chipid: [u8; 2] = [0; 2];
        self.blocking_sensor_writereg8_8(0xFF, 0x01)?;
        self.sensor_readreg8_8(OV2640_CHIPID_HIGH, &mut chipid[0..1])?;
        self.sensor_readreg8_8(OV2640_CHIPID_LOW, &mut chipid[1..2])?;
        Ok(chipid)
    }
}
