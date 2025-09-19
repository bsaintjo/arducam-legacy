use core::marker::PhantomData;

use embedded_hal_1::{delay::DelayNs, spi::Operation};

use crate::{ArducamError, Resolution, ARDUCHIP_TRIG, CAP_DONE_MASK, FIFO_BURST};

const READ_FLAG: u8 = 0x7f;
const WRITE_FLAG: u8 = 0x80;

struct OV5642;

pub struct Arducam<C, M, I, S> {
    camera: C,
    mode: PhantomData<M>,
    pub i2c: I,
    pub spi: S,
    resolution: Resolution,
}

struct OV2640;

pub trait Camera {
    type I2cRegisterAddr;
    type MultiRegister;
}

impl Camera for OV2640 {
    type I2cRegisterAddr = u8;
    type MultiRegister = [u8; 2];
}

pub trait BlockingCamera: Camera {
    fn i2c_read(&mut self, reg: Self::I2cRegisterAddr, out: &mut u8) -> Result<(), ArducamError>;

    fn i2c_write(&mut self, reg: Self::I2cRegisterAddr, data: u8) -> Result<(), ArducamError>;

    fn i2c_write_registers(&mut self, regs: &[Self::MultiRegister]) -> Result<(), ArducamError>;

    fn spi_write<D: DelayNs>(&mut self, data: &mut [u8], delay: D) -> Result<(), ArducamError>;

    fn spi_read(&mut self, addr: u8) -> Result<u8, ArducamError>;
    fn spi_read_values(&mut self, out: &mut [u8]) -> Result<(), ArducamError>;

    fn init(&mut self) -> Result<(), ArducamError>;

    fn flush_fifo<D: DelayNs>(&mut self, delay: D) -> Result<(), ArducamError> {
        self.spi_write(&mut [0x04, 0x01], delay)
    }

    fn clear_fifo_flag<D: DelayNs>(&mut self, delay: D) -> Result<(), ArducamError> {
        self.spi_write(&mut [0x04, 0x01], delay)
    }

    fn start_capture<D: DelayNs>(&mut self, delay: D) -> Result<(), ArducamError> {
        self.spi_write(&mut [0x04, 0x02], delay)
    }

    fn is_capture_done(&mut self) -> Result<bool, ArducamError> {
        self.spi_read(ARDUCHIP_TRIG)
            .map(|result| result & CAP_DONE_MASK != 0)
    }

    fn read_captured_image<D: DelayNs>(&mut self, out: &mut [u8], delay: D) -> Result<(), ArducamError> {
        self.spi_write(&mut [FIFO_BURST], delay)?;
        self.spi_read_values(out)?;
        Ok(())
        // self.spi
        //     .transaction(&mut [
        //         Operation::Write(&[FIFO_BURST & 0x7F]),
        //         Operation::Read(out),
        //         Operation::Write(&[ARDUCHIP_FIFO | 0x80, FIFO_CLEAR_MASK]),
        //     ])
            // .map_err(|e| ArducamError::SpiError(e.kind()))
    }
}

pub trait AsyncCamera: Camera {

    async fn i2c_read(&mut self, reg: Self::I2cRegisterAddr, out: &mut [u8]) -> Result<(), ArducamError>;

    async fn i2c_write(&mut self, reg: Self::I2cRegisterAddr, data: u8) -> Result<(), ArducamError>;

    async fn spi_write(&mut self, addr: Self::I2cRegisterAddr, data: u8) -> Result<(), ArducamError>;

    async fn spi_read(&mut self, addr: Self::I2cRegisterAddr) -> Result<u8, ArducamError>;

    async fn i2c_write_registers(&mut self, regs: &[[u8; 2]]) -> Result<(), ArducamError>;

    async fn send_resolution(&mut self) -> Result<(), ArducamError>;
    // async fn send_resolution(&mut self) -> Result<(), ArducamError> {
    //     match self.resolution {
    //         Resolution::Res160x120 => self.sensor_writeregs8_8(&OV2640_160x120_JPEG).await,
    //         Resolution::Res1024x768 => self.sensor_writeregs8_8(&OV2640_1024x768_JPEG).await,
    //         Resolution::Res1280x1024 => self.sensor_writeregs8_8(&OV2640_1280x1024_JPEG).await,
    //         Resolution::Res1600x1200 => self.sensor_writeregs8_8(&OV2640_1600x1200_JPEG).await,
    //         Resolution::Res176x144 => self.sensor_writeregs8_8(&OV2640_176x144_JPEG).await,
    //         Resolution::Res320x240 => self.sensor_writeregs8_8(&OV2640_320x240_JPEG).await,
    //         Resolution::Res352x288 => self.sensor_writeregs8_8(&OV2640_352x288_JPEG).await,
    //         Resolution::Res640x480 => self.sensor_writeregs8_8(&OV2640_640x480_JPEG).await,
    //         Resolution::Res800x600 => self.sensor_writeregs8_8(&OV2640_800x600_JPEG).await,
    //     }
    // }

    async fn flush_fifo(&mut self) -> Result<(), ArducamError>;
        // self.arduchip_write_reg(ARDUCHIP_FIFO, FIFO_CLEAR_MASK)
        //     .await
    // }

    async fn start_fifo(&mut self) -> Result<(), ArducamError>;
    // async fn start_fifo(&mut self) -> Result<(), ArducamError> {
        // self.arduchip_write_reg(ARDUCHIP_FIFO, FIFO_START_MASK)
        //     .await
    // }

    // async fn init(&mut self) -> Result<(), ArducamError>;
        // self.arduchip_write_reg(0x07, 0x80).await?;
        // self.transaction(&mut [Operation::DelayNs(100_000_000)])
        //     .await?;
        // self.arduchip_write_reg(0x07, 0x00).await?;
        // self.transaction(&mut [Operation::DelayNs(100_000_000)])
        //     .await?;

        // self.sensor_writereg8_8(0xFF, 0x01).await?;
        // self.transaction(&mut [Operation::DelayNs(100_000_000)])
        //     .await?;

        // self.sensor_writereg8_8(0x12, 0x80).await?;
        // self.transaction(&mut [Operation::DelayNs(100_000_000)])
        //     .await?;

        // self.sensor_writeregs8_8(&OV2640_JPEG_INIT).await?;
        // self.sensor_writeregs8_8(&OV2640_YUV422).await?;
        // self.sensor_writeregs8_8(&OV2640_JPEG).await?;
        // self.sensor_writereg8_8(0xFF, 0x01).await?;
        // self.sensor_writereg8_8(0x15, 0x00).await?;
        // self.send_resolution().await?;

    //     Ok(())
    // }
}