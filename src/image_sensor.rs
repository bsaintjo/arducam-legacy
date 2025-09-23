use embedded_hal_1::i2c::I2c;

enum MirrorFlip {
    Original,
    Mirrored,
    Flipped,
    MirrorFlipped,
}

impl MirrorFlip {
    fn to_command(&self) -> u8 {
        const DEFAULT_VALUE: u8 = 0x80;
        match self {
            MirrorFlip::Original => DEFAULT_VALUE,
            MirrorFlip::Mirrored => DEFAULT_VALUE | (0b10 << 5),
            MirrorFlip::Flipped => DEFAULT_VALUE | (0b01 << 5),
            MirrorFlip::MirrorFlipped => DEFAULT_VALUE | (0b11 << 5),
        }
    }

    fn control<I: I2c>(self, i2c: &mut I) {
        i2c.write(0x30, &[0x38, 0x18, self.to_command()]).unwrap();
        if matches!(self, MirrorFlip::Mirrored | MirrorFlip::MirrorFlipped) {
            i2c.write(0x30, &[0x36, 0x21, self.to_command()]).unwrap();
            i2c.write(0x30, &[0x38, 0x01, self.to_command()]).unwrap();
        }
    }
}

/// color bar
/// Register: 0x503D
/// Bit[7]: color bar enable
/// 0: color bar OFF
/// 1: color bar enable
/// Bit[5:4]: color bar pattern select
/// 00: color bar pattern
enum TestPattern {
    Disable,
    Pattern1,
    Pattern2,
    Pattern3,
    Pattern4,
}

impl TestPattern {
    fn to_command(&self) -> u8 {
        const DEFAULT_VALUE: u8 = 0x0;
        match self {
            TestPattern::Disable => DEFAULT_VALUE,
            TestPattern::Pattern1 => 0b01000000,
            TestPattern::Pattern2 => 0b01010000,
            TestPattern::Pattern3 => 0b01001000,
            TestPattern::Pattern4 => 0b01011000,
        };
        todo!()
    }
    fn control<I: I2c>(self, i2c: &mut I) {}
}

enum DigitalGain {
    Automatic,
    X1,
    X2,
    X3,
    X4,
}

trait ToCommand {
    const DEFAULT_VALUE: u8;
    const ADDRESS: u16;
}
