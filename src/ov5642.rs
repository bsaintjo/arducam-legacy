use core::marker::PhantomData;

use embedded_hal_1::{
    delay::DelayNs,
    i2c::{Error as I2cError, I2c},
    spi::{Error as SpiError, Operation, SpiDevice},
};
use embedded_hal_async::{i2c::I2c as AsyncI2c, spi::SpiDevice as AsyncSpiDevice};

use crate::{
    ov5642_registers::{
        OV5642_1280_X_960_RAW, OV5642_320_X_240, OV5642_640_X_480_RAW, OV5642_JPEG_CAPTURE_QSXGA,
        OV5642_QVGA_PREVIEW, OV5642_QVGA_PREVIEW_1, OV5642_QVGA_PREVIEW_2,
    },
    ArducamError, Resolution, ARDUCHIP_FIFO, ARDUCHIP_TEST1, ARDUCHIP_TRIG, CAP_DONE_MASK,
    FIFO_BURST, FIFO_CLEAR_MASK,
};

const I2C_ADDR: u8 = 0x3c;
const OV562_CHIPID_HIGH_ADDR: [u8; 2] = [0x30, 0x0a];
const OV562_CHIPID_LOW_ADDR: [u8; 2] = [0x30, 0x0b];
pub const OV562_CHIPID: u16 = 0x5642;

pub enum CameraMode {
    JPEG,
    RAW,
    BMP,
}

struct Blocking;
struct Async;

pub struct Arducam5MPConfig {
    pub mode: CameraMode,
}

pub struct Arducam5MP<M, I, S> {
    mode: PhantomData<M>,
    pub(crate) i2c: I,
    pub(crate) spi: S,
    config: Arducam5MPConfig,
}

impl<I, S> Arducam5MP<Blocking, I, S>
where
    I: I2c,
    S: SpiDevice,
{
    pub fn new(i2c: I, spi: S, config: Arducam5MPConfig) -> Self {
        Self {
            i2c,
            spi,
            config,
            mode: PhantomData,
        }
    }

    fn spi_write<D: DelayNs>(
        &mut self,
        addr: u8,
        value: Option<u8>,
        mut delay: D,
    ) -> Result<(), ArducamError> {
        const WRITE_FLAG: u8 = 0x80;
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
        const READ_FLAG: u8 = 0x7f;
        let value = &mut [0u8];
        self.spi
            .transaction(&mut [
                Operation::Write(&[addr & READ_FLAG]),
                Operation::Read(value),
            ])
            .map_err(|e| ArducamError::SpiError(e.kind()))?;
        Ok(value[0])
    }

    fn i2c_write(&mut self, addr: u16, value: u8) -> Result<(), ArducamError> {
        let mut buffer = [0u8; 3];
        buffer[0..2].copy_from_slice(&addr.to_be_bytes());
        buffer[2] = value;
        self.i2c
            .write(I2C_ADDR, &buffer)
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
            .write_read(I2C_ADDR, &addr.to_be_bytes(), core::slice::from_mut(output))
            .map_err(|e| ArducamError::I2cError(e.kind()))
    }

    pub fn spi_test<D: DelayNs>(&mut self, delay: D) -> Result<bool, ArducamError> {
        let test_value = 0x56;
        self.spi_write(ARDUCHIP_TEST1, Some(test_value), delay)?;
        let result = self.spi_read(ARDUCHIP_TEST1)?;
        Ok(test_value == result)
    }

    pub fn chip_id(&mut self, chip_id: &mut u16) -> Result<(), ArducamError> {
        let mut tmp_chip_id = [0u8; 2];
        self.i2c_write(0x00FF, 0x01)?;
        self.i2c_read(
            u16::from_be_bytes(OV562_CHIPID_HIGH_ADDR),
            &mut tmp_chip_id[0],
        )?;
        self.i2c_read(
            u16::from_be_bytes(OV562_CHIPID_LOW_ADDR),
            &mut tmp_chip_id[1],
        )?;
        *chip_id = u16::from_be_bytes(tmp_chip_id);
        Ok(())
    }

    pub fn init<D: DelayNs>(&mut self, mut delay: D) -> Result<(), ArducamError> {
        // self.spi_write(0x07, 0x80, &mut delay)?;
        // delay.delay_ms(100);
        // self.spi_write(0x07, 0x00, &mut delay)?;
        // delay.delay_ms(100);

        self.i2c_write(0x3008, 0x80)?;
        self.i2c_write_registers(&OV5642_QVGA_PREVIEW, &mut delay)?;
        match self.config.mode {
            CameraMode::JPEG => {
                // self.i2c_write_registers(&OV5642_QVGA_PREVIEW_1)?;
                // self.i2c_write_registers(&OV5642_QVGA_PREVIEW_2)?;
                delay.delay_ns(100);
                delay.delay_ns(100);
                self.i2c_write_registers(&OV5642_JPEG_CAPTURE_QSXGA, &mut delay)?;
                self.i2c_write_registers(&OV5642_320_X_240, &mut delay)?;
                delay.delay_ns(100);
                self.i2c_write(0x3818, 0xa8)?;
                self.i2c_write(0x3621, 0x10)?;
                self.i2c_write(0x3801, 0xb0)?;
                self.i2c_write(0x4407, 0x04)?;
            }

            CameraMode::BMP => {
                // self.i2c_write_registers(&OV5642_QVGA_PREVIEW_1)?;
                // self.i2c_write_registers(&OV5642_QVGA_PREVIEW_2)?;
                delay.delay_ns(100);
                self.i2c_write(0x4740, 0x21)?;
                self.i2c_write(0x501e, 0x2a)?;
                self.i2c_write(0x5002, 0xf8)?;
                self.i2c_write(0x501f, 0x01)?;
                self.i2c_write(0x4300, 0x61)?;

                let mut reg_val = 0u8;
                self.i2c_read(0x3818, &mut reg_val)?;
                self.i2c_write(0x3818, reg_val | 0x60)?;

                self.i2c_read(0x3621, &mut reg_val)?;
                self.i2c_write(0x3621, reg_val & 0xdf)?;
            }

            CameraMode::RAW => {
                self.i2c_write_registers(&OV5642_1280_X_960_RAW, &mut delay)?;
                self.i2c_write_registers(&OV5642_640_X_480_RAW, &mut delay)?;
            }
        }
        Ok(())
    }

    pub fn flush_fifo<D: DelayNs>(&mut self, delay: D) -> Result<(), ArducamError> {
        self.spi_write(0x04, Some(0x01), delay)
    }

    pub fn clear_fifo_flag<D: DelayNs>(&mut self, delay: D) -> Result<(), ArducamError> {
        self.spi_write(0x04, Some(0x01), delay)
    }

    pub fn start_capture<D: DelayNs>(&mut self, delay: D) -> Result<(), ArducamError> {
        self.spi_write(0x04, Some(0x02), delay)
    }

    pub fn is_capture_done(&mut self) -> Result<bool, ArducamError> {
        self.spi_read(ARDUCHIP_TRIG)
            .map(|result| result & CAP_DONE_MASK != 0)
    }

    pub fn read_captured_image(&mut self, out: &mut [u8]) -> Result<(), ArducamError> {
        self.spi
            .transaction(&mut [
                Operation::Write(&[FIFO_BURST & 0x7F]),
                Operation::Read(out),
                Operation::Write(&[ARDUCHIP_FIFO | 0x80, FIFO_CLEAR_MASK]),
            ])
            .map_err(|e| ArducamError::SpiError(e.kind()))
    }

    pub fn fifo_length(&mut self, fifo_length: &mut u32) -> Result<(), ArducamError> {
        let fst = self.spi_read(0x42)?;
        let snd = self.spi_read(0x43)?;
        let thrd = self.spi_read(0x44)?;
        // *fifo_length = u32::from_be_bytes([0u8, thrd, snd, fst]);
        *fifo_length = ((thrd as u32) << 16) | ((snd as u32) << 8) | (fst as u32);
        *fifo_length &= 0x07fffff;
        Ok(())
    }

    pub fn vsync_mask<D: DelayNs>(&mut self, delay: D) -> Result<(), ArducamError> {
        const ARDUCHIP_TIM: u8 = 0x03;
        const VSYNC_LEVEL_MASK: u8 = 0x02;
        self.spi_write(ARDUCHIP_TIM, Some(VSYNC_LEVEL_MASK), delay)
    }

    pub fn frames<D: DelayNs>(&mut self, delay: D) -> Result<(), ArducamError> {
        const ARDUCHIP_FRAMES: u8 = 0x01;
        self.spi_write(ARDUCHIP_FRAMES, Some(0x00), delay)
    }

    pub fn set_jpeg_size<D: DelayNs>(
        &mut self,
        resolution: Resolution,
        mut delay: D,
    ) -> Result<(), ArducamError> {
        match resolution {
            Resolution::Res160x120 => self.i2c_write_registers(&OV5642_320_X_240, &mut delay),
            Resolution::Res1024x768 => self.i2c_write_registers(&OV5642_320_X_240, &mut delay),
            Resolution::Res1280x1024 => self.i2c_write_registers(&OV5642_320_X_240, &mut delay),
            Resolution::Res1600x1200 => self.i2c_write_registers(&OV5642_320_X_240, &mut delay),
            Resolution::Res176x144 => self.i2c_write_registers(&OV5642_320_X_240, &mut delay),
            Resolution::Res320x240 => self.i2c_write_registers(&OV5642_320_X_240, &mut delay),
            Resolution::Res352x288 => self.i2c_write_registers(&OV5642_320_X_240, &mut delay),
            Resolution::Res640x480 => self.i2c_write_registers(&OV5642_320_X_240, &mut delay),
            Resolution::Res800x600 => self.i2c_write_registers(&OV5642_320_X_240, &mut delay),
        }?;
        Ok(())
    }
}

impl<I, S> Arducam5MP<Async, I, S>
where
    I: AsyncI2c,
    S: AsyncSpiDevice,
{
    pub fn new_async(i2c: I, spi: S, config: Arducam5MPConfig) -> Self {
        Self {
            i2c,
            spi,
            config,
            mode: PhantomData,
        }
    }

    async fn spi_write<D: DelayNs>(
        &mut self,
        addr: u8,
        value: u8,
        mut delay: D,
    ) -> Result<(), ArducamError> {
        const WRITE_FLAG: u8 = 0x80;
        let addr = addr | WRITE_FLAG;
        self.spi
            .write(&[addr, value])
            .await
            .map_err(|e| ArducamError::SpiError(e.kind()))?;
        delay.delay_ms(1);
        Ok(())
    }

    async fn spi_read(&mut self, addr: u8) -> Result<u8, ArducamError> {
        const READ_FLAG: u8 = 0x7f;
        let value = &mut [0u8];
        self.spi
            .transaction(&mut [
                Operation::Write(&[addr & READ_FLAG]),
                Operation::Read(value),
            ])
            .await
            .map_err(|e| ArducamError::SpiError(e.kind()))?;
        Ok(value[0])
    }

    async fn i2c_write(&mut self, addr: u16, value: u8) -> Result<(), ArducamError> {
        let mut buffer = [0u8; 3];
        buffer[0..2].copy_from_slice(&addr.to_be_bytes());
        buffer[2] = value;
        self.i2c
            .write(I2C_ADDR, &buffer)
            .await
            .map_err(|e| ArducamError::I2cError(e.kind()))
    }

    async fn i2c_write_registers<D: DelayNs>(
        &mut self,
        regs: &[[u8; 3]],
        mut delay: D,
    ) -> Result<(), ArducamError> {
        for reg in regs {
            let addr = u16::from_be_bytes([reg[0], reg[1]]);
            self.i2c_write(addr, reg[2]).await?;
            delay.delay_ms(1);
        }
        Ok(())
    }

    async fn i2c_read(&mut self, addr: u16, output: &mut u8) -> Result<(), ArducamError> {
        self.i2c
            .write_read(I2C_ADDR, &addr.to_be_bytes(), core::slice::from_mut(output))
            .await
            .map_err(|e| ArducamError::I2cError(e.kind()))
    }

    pub async fn spi_test<D: DelayNs>(&mut self, delay: D) -> Result<bool, ArducamError> {
        let test_value = 0x56;
        self.spi_write(ARDUCHIP_TEST1, test_value, delay).await?;
        let result = self.spi_read(ARDUCHIP_TEST1).await?;
        Ok(test_value == result)
    }

    pub async fn chip_id(&mut self, chip_id: &mut u16) -> Result<(), ArducamError> {
        let mut tmp_chip_id = [0u8; 2];
        self.i2c_write(0x00FF, 0x01).await?;
        self.i2c_read(
            u16::from_be_bytes(OV562_CHIPID_HIGH_ADDR),
            &mut tmp_chip_id[0],
        )
        .await?;
        self.i2c_read(
            u16::from_be_bytes(OV562_CHIPID_LOW_ADDR),
            &mut tmp_chip_id[1],
        )
        .await?;
        *chip_id = u16::from_be_bytes(tmp_chip_id);
        Ok(())
    }

    pub async fn init<D: DelayNs>(&mut self, mut delay: D) -> Result<(), ArducamError> {
        // self.spi_write(0x07, 0x80, &mut delay)?;
        // delay.delay_ms(100);
        // self.spi_write(0x07, 0x00, &mut delay)?;
        // delay.delay_ms(100);

        self.i2c_write(0x3008, 0x80).await?;
        self.i2c_write_registers(&OV5642_QVGA_PREVIEW, &mut delay)
            .await?;
        match self.config.mode {
            CameraMode::JPEG => {
                // self.i2c_write_registers(&OV5642_QVGA_PREVIEW_1)?;
                // self.i2c_write_registers(&OV5642_QVGA_PREVIEW_2)?;
                delay.delay_ns(100);
                delay.delay_ns(100);
                self.i2c_write_registers(&OV5642_JPEG_CAPTURE_QSXGA, &mut delay)
                    .await?;
                self.i2c_write_registers(&OV5642_320_X_240, &mut delay)
                    .await?;
                delay.delay_ns(100);
                self.i2c_write(0x3818, 0xa8).await?;
                self.i2c_write(0x3621, 0x10).await?;
                self.i2c_write(0x3801, 0xb0).await?;
                self.i2c_write(0x4407, 0x04).await?;
            }

            CameraMode::BMP => {
                // self.i2c_write_registers(&OV5642_QVGA_PREVIEW_1)?;
                // self.i2c_write_registers(&OV5642_QVGA_PREVIEW_2)?;
                delay.delay_ns(100);
                self.i2c_write(0x4740, 0x21).await?;
                self.i2c_write(0x501e, 0x2a).await?;
                self.i2c_write(0x5002, 0xf8).await?;
                self.i2c_write(0x501f, 0x01).await?;
                self.i2c_write(0x4300, 0x61).await?;

                let mut reg_val = 0u8;
                self.i2c_read(0x3818, &mut reg_val).await?;
                self.i2c_write(0x3818, reg_val | 0x60).await?;

                self.i2c_read(0x3621, &mut reg_val).await?;
                self.i2c_write(0x3621, reg_val & 0xdf).await?;
            }

            CameraMode::RAW => {
                self.i2c_write_registers(&OV5642_1280_X_960_RAW, &mut delay)
                    .await?;
                self.i2c_write_registers(&OV5642_640_X_480_RAW, &mut delay)
                    .await?;
            }
        }
        Ok(())
    }

    pub async fn flush_fifo<D: DelayNs>(&mut self, delay: D) -> Result<(), ArducamError> {
        self.spi_write(0x04, 0x01, delay).await
    }

    pub async fn clear_fifo_flag<D: DelayNs>(&mut self, delay: D) -> Result<(), ArducamError> {
        self.spi_write(0x04, 0x01, delay).await
    }

    pub async fn start_capture<D: DelayNs>(&mut self, delay: D) -> Result<(), ArducamError> {
        self.spi_write(0x04, 0x02, delay).await
    }

    pub async fn is_capture_done(&mut self) -> Result<bool, ArducamError> {
        self.spi_read(ARDUCHIP_TRIG)
            .await
            .map(|result| result & CAP_DONE_MASK != 0)
    }

    pub async fn read_captured_image(&mut self, out: &mut [u8]) -> Result<(), ArducamError> {
        self.spi
            .transaction(&mut [
                Operation::Write(&[FIFO_BURST & 0x7F]),
                Operation::Read(out),
                Operation::Write(&[ARDUCHIP_FIFO | 0x80, FIFO_CLEAR_MASK]),
            ])
            .await
            .map_err(|e| ArducamError::SpiError(e.kind()))
    }

    pub async fn fifo_length(&mut self, fifo_length: &mut u32) -> Result<(), ArducamError> {
        let fst = self.spi_read(0x42).await?;
        let snd = self.spi_read(0x43).await?;
        let thrd = self.spi_read(0x44).await?;
        // *fifo_length = u32::from_be_bytes([0u8, thrd, snd, fst]);
        *fifo_length = ((thrd as u32) << 16) | ((snd as u32) << 8) | (fst as u32);
        *fifo_length &= 0x07fffff;
        Ok(())
    }

    pub async fn vsync_mask<D: DelayNs>(&mut self, delay: D) -> Result<(), ArducamError> {
        const ARDUCHIP_TIM: u8 = 0x03;
        const VSYNC_LEVEL_MASK: u8 = 0x02;
        self.spi_write(ARDUCHIP_TIM, VSYNC_LEVEL_MASK, delay).await
    }

    pub async fn frames<D: DelayNs>(&mut self, delay: D) -> Result<(), ArducamError> {
        const ARDUCHIP_FRAMES: u8 = 0x01;
        self.spi_write(ARDUCHIP_FRAMES, 0x00, delay).await
    }

    pub async fn set_jpeg_size<D: DelayNs>(
        &mut self,
        resolution: Resolution,
        mut delay: D,
    ) -> Result<(), ArducamError> {
        match resolution {
            Resolution::Res160x120 => self.i2c_write_registers(&OV5642_320_X_240, &mut delay),
            Resolution::Res1024x768 => self.i2c_write_registers(&OV5642_320_X_240, &mut delay),
            Resolution::Res1280x1024 => self.i2c_write_registers(&OV5642_320_X_240, &mut delay),
            Resolution::Res1600x1200 => self.i2c_write_registers(&OV5642_320_X_240, &mut delay),
            Resolution::Res176x144 => self.i2c_write_registers(&OV5642_320_X_240, &mut delay),
            Resolution::Res320x240 => self.i2c_write_registers(&OV5642_320_X_240, &mut delay),
            Resolution::Res352x288 => self.i2c_write_registers(&OV5642_320_X_240, &mut delay),
            Resolution::Res640x480 => self.i2c_write_registers(&OV5642_320_X_240, &mut delay),
            Resolution::Res800x600 => self.i2c_write_registers(&OV5642_320_X_240, &mut delay),
        }
        .await?;
        Ok(())
    }
}
