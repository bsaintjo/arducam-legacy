use core::marker::PhantomData;

use embedded_hal_1::{
    delay::DelayNs,
    i2c::{Error as I2cError, I2c},
    spi::{Error as SpiError, Operation, SpiDevice},
};

use crate::{
    ov5642::Arducam5MP, ArducamError, Blocking, ARDUCHIP_TRIG, CAP_DONE_MASK,
};

const READ_FLAG: u8 = 0x7f;
const WRITE_FLAG: u8 = 0x80;


pub struct Sensor<'a, T, C> {
    i2c: &'a mut T,
    camera: PhantomData<C>,
}

impl<'a, I: I2c, C: BlockingCamera> Sensor<'a, I, C> {
    fn i2c_write(&mut self, addr: u16, value: u8) -> Result<(), ArducamError> {
        let mut buffer = [0u8; 3];
        buffer[0..2].copy_from_slice(&addr.to_be_bytes());
        buffer[2] = value;
        self.i2c
            .write(C::I2C_ADDR, &buffer)
            .map_err(|e| ArducamError::I2cError(e.kind()))
    }

    fn i2c_write_registers<D: DelayNs>(
        &mut self,
        regs: &[[u8; 3]],
        mut delay: D,
    ) -> Result<(), ArducamError> {
        for reg in regs {
            let addr = u16::from_be_bytes([reg[0], reg[1]]);
            self.i2c_write(addr, reg[2])?;
            delay.delay_ms(1);
        }
        Ok(())
    }

    fn i2c_read(&mut self, addr: u16, output: &mut u8) -> Result<(), ArducamError> {
        self.i2c
            .write_read(
                C::I2C_ADDR,
                &addr.to_be_bytes(),
                core::slice::from_mut(output),
            )
            .map_err(|e| ArducamError::I2cError(e.kind()))
    }
}
pub struct Arduchip<'a, T> {
    spi: &'a mut T,
}

impl<'a, I: SpiDevice> Arduchip<'a, I> {
    fn spi_write<D: DelayNs>(
        &mut self,
        addr: u8,
        value: Option<u8>,
        mut delay: D,
    ) -> Result<(), ArducamError> {
        let addr = addr | WRITE_FLAG;
        let buf = {
            if let Some(value) = value {
                &[addr, value] as &[u8]
            } else {
                &[addr] as &[u8]
            }
        };
        self.spi
            .write(buf)
            .map_err(|e| ArducamError::SpiError(e.kind()))?;
        delay.delay_ms(1);
        Ok(())
    }

    fn spi_read(&mut self, addr: u8) -> Result<u8, ArducamError> {
        let value = &mut [0u8];
        self.spi
            .transaction(&mut [
                Operation::Write(&[addr & READ_FLAG]),
                Operation::Read(value),
            ])
            .map_err(|e| ArducamError::SpiError(e.kind()))?;
        Ok(value[0])
    }
}

pub trait BlockingCamera: Sized {
    const I2C_ADDR: u8;
    type I: I2c;
    type S: SpiDevice;
    fn sensor(&mut self) -> Sensor<'_, Self::I, Self>;
    fn arduchip(&mut self) -> Arduchip<'_, Self::S>;

    fn flush_fifo<D: DelayNs>(&mut self, delay: D) -> Result<(), ArducamError> {
        self.arduchip().spi_write(0x04, Some(0x01), delay)
    }

    fn clear_fifo_flag<D: DelayNs>(&mut self, delay: D) -> Result<(), ArducamError> {
        self.arduchip().spi_write(0x04, Some(0x01), delay)
    }

    fn start_capture<D: DelayNs>(&mut self, delay: D) -> Result<(), ArducamError> {
        self.arduchip().spi_write(0x04, Some(0x02), delay)
    }

    fn is_capture_done(&mut self) -> Result<bool, ArducamError> {
        self.arduchip()
            .spi_read(ARDUCHIP_TRIG)
            .map(|result| result & CAP_DONE_MASK != 0)
    }
}

impl<I: I2c, S: SpiDevice> BlockingCamera for Arducam5MP<Blocking, I, S> {
    const I2C_ADDR: u8 = 0x3c;
    type I = I;
    type S = S;
    fn sensor(&mut self) -> Sensor<'_, Self::I, Self> {
        Sensor {
            i2c: &mut self.i2c,
            camera: PhantomData,
        }
    }

    fn arduchip(&mut self) -> Arduchip<'_, Self::S> {
        Arduchip {
            spi: &mut self.spi,
        }
    }
}

// pub trait AsyncCamera: Camera {
//     async fn i2c_read(
//         &mut self,
//         reg: Self::I2cRegisterAddr,
//         out: &mut [u8],
//     ) -> Result<(), ArducamError>;

//     async fn i2c_write(&mut self, reg: Self::I2cRegisterAddr, data: u8)
//         -> Result<(), ArducamError>;

//     async fn spi_write(
//         &mut self,
//         addr: Self::I2cRegisterAddr,
//         data: u8,
//     ) -> Result<(), ArducamError>;

//     async fn spi_read(&mut self, addr: Self::I2cRegisterAddr) -> Result<u8, ArducamError>;

//     async fn i2c_write_registers(&mut self, regs: &[[u8; 2]]) -> Result<(), ArducamError>;

//     async fn send_resolution(&mut self) -> Result<(), ArducamError>;
//     // async fn send_resolution(&mut self) -> Result<(), ArducamError> {
//     //     match self.resolution {
//     //         Resolution::Res160x120 => self.sensor_writeregs8_8(&OV2640_160x120_JPEG).await,
//     //         Resolution::Res1024x768 => self.sensor_writeregs8_8(&OV2640_1024x768_JPEG).await,
//     //         Resolution::Res1280x1024 => self.sensor_writeregs8_8(&OV2640_1280x1024_JPEG).await,
//     //         Resolution::Res1600x1200 => self.sensor_writeregs8_8(&OV2640_1600x1200_JPEG).await,
//     //         Resolution::Res176x144 => self.sensor_writeregs8_8(&OV2640_176x144_JPEG).await,
//     //         Resolution::Res320x240 => self.sensor_writeregs8_8(&OV2640_320x240_JPEG).await,
//     //         Resolution::Res352x288 => self.sensor_writeregs8_8(&OV2640_352x288_JPEG).await,
//     //         Resolution::Res640x480 => self.sensor_writeregs8_8(&OV2640_640x480_JPEG).await,
//     //         Resolution::Res800x600 => self.sensor_writeregs8_8(&OV2640_800x600_JPEG).await,
//     //     }
//     // }

//     async fn flush_fifo(&mut self) -> Result<(), ArducamError>;
//     // self.arduchip_write_reg(ARDUCHIP_FIFO, FIFO_CLEAR_MASK)
//     //     .await
//     // }

//     async fn start_fifo(&mut self) -> Result<(), ArducamError>;
//     // async fn start_fifo(&mut self) -> Result<(), ArducamError> {
//     // self.arduchip_write_reg(ARDUCHIP_FIFO, FIFO_START_MASK)
//     //     .await
//     // }

//     // async fn init(&mut self) -> Result<(), ArducamError>;
//     // self.arduchip_write_reg(0x07, 0x80).await?;
//     // self.transaction(&mut [Operation::DelayNs(100_000_000)])
//     //     .await?;
//     // self.arduchip_write_reg(0x07, 0x00).await?;
//     // self.transaction(&mut [Operation::DelayNs(100_000_000)])
//     //     .await?;

//     // self.sensor_writereg8_8(0xFF, 0x01).await?;
//     // self.transaction(&mut [Operation::DelayNs(100_000_000)])
//     //     .await?;

//     // self.sensor_writereg8_8(0x12, 0x80).await?;
//     // self.transaction(&mut [Operation::DelayNs(100_000_000)])
//     //     .await?;

//     // self.sensor_writeregs8_8(&OV2640_JPEG_INIT).await?;
//     // self.sensor_writeregs8_8(&OV2640_YUV422).await?;
//     // self.sensor_writeregs8_8(&OV2640_JPEG).await?;
//     // self.sensor_writereg8_8(0xFF, 0x01).await?;
//     // self.sensor_writereg8_8(0x15, 0x00).await?;
//     // self.send_resolution().await?;

//     //     Ok(())
//     // }
// }
