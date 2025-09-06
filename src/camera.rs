use core::marker::PhantomData;

use embedded_hal_1::spi::Operation;

use crate::{ArducamError, Resolution};

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
    const READ_FLAG: u8;
    const WRITE_FLAG: u8;
    type RegisterType;
}

impl Camera for OV2640 {
    const READ_FLAG: u8 = 0x7f;
    const WRITE_FLAG: u8 = 0x80;
    type RegisterType = u8;
}

pub trait AsyncCamera: Camera {

    async fn sensor_readreg8_8(&mut self, reg: Self::RegisterType, out: &mut [u8]) -> Result<(), ArducamError>;

    async fn sensor_writereg8_8(&mut self, reg: Self::RegisterType, data: u8) -> Result<(), ArducamError>;
    // async fn sensor_writereg8_8(&mut self, reg: u8, data: u8) -> Result<(), ArducamError> {
        // self.i2c
        //     .write(OV2640_ADDR, &[reg, data])
        //     .await
        //     .map_err(|e| ArducamError::I2cError(e.kind()))
    // }

    async fn arduchip_write_reg(&mut self, addr: Self::RegisterType, data: u8) -> Result<(), ArducamError>;
        // self.arduchip_write(addr | 0x80, data)
        // self.spi
        //     .write(&[addr | 0x80, data])
        //     .await
        //     .map_err(|e| ArducamError::SpiError(e.kind()))?;
        // Ok(())
        // self.spi
        //     .transaction(&mut [Operation::Write(&[addr | 0x80]), Operation::Write(&[data])])
        //     .await
        //     .map_err(|e| ArducamError::SpiError(e.kind()))?;
        // Ok(())
    // }

    async fn transaction(
        &mut self,
        operations: &mut [Operation<'_, u8>],
    ) -> Result<(), ArducamError>;
    // async fn transaction(
    //     &mut self,
    //     operations: &mut [Operation<'_, u8>],
    // ) -> Result<(), ArducamError> {
    //     self.spi
    //         .transaction(operations)
    //         .await
    //         .map_err(|e| ArducamError::SpiError(e.kind()))
    // }

    async fn arduchip_read_reg(&mut self, addr: Self::RegisterType) -> Result<u8, ArducamError>;
    // async fn arduchip_read_reg(&mut self, addr: u8) -> Result<u8, ArducamError> {
        // self.arduchip_read(addr & 0x7F)
        // let mut value = [0u8; 1];
        // self.transaction(&mut [
        //     Operation::Write(&[addr & 0x7f]),
        //     Operation::Read(&mut value),
        // ])
        // .await?;
        // Ok(value[0])
    // }

    async fn sensor_writeregs8_8(&mut self, regs: &[[u8; 2]]) -> Result<(), ArducamError>;
    // async fn sensor_writeregs8_8(&mut self, regs: &[[u8; 2]]) -> Result<(), ArducamError> {
        // for reg in regs {
        //     self.sensor_writereg8_8(reg[0], reg[1]).await?;
        // }
        // Ok(())
    // }

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

    async fn init(&mut self) -> Result<(), ArducamError>;
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