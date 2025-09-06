[arducam]: https://docs.arducam.com/Arduino-SPI-camera/Legacy-SPI-camera/Introduction/
[embedded-hal]: https://github.com/rust-embedded/embedded-hal

# `arducam-legacy`

This library is a fork of the original repo. This fork adds support for [embedded-hal] version 1.0 and has been tested on a STM32 development board.

This Rust library provides [embedded-hal][embedded-hal] support for [legacy Arducam SPI-based cameras][arducam] like e.g. ArduCAM MINI 2MP Plus. It has been rewritten from original Arducam library written in C.

## Milestones

- [x] OV2640 support
- [x] JPEG image support
- [x] Different resolutions
- [ ] More sensor models support
- [ ] DMA API

## Model support

Currently this library is only tested with Arducam Mini 2MP Plus model. Contributions for other models are welcome!

## MCU support

Any microcontroller HAL with [embedded-hal][embedded-hal] support should work with this driver

## Usage

Add this line to `Cargo.toml` under ```[dependencies]``` of your project

```toml
[dependencies]
arducam-legacy = "0.2.0"
```

## Example

There are included examples for the [STM32U083RC](examples/stm32u083rc/) and [Raspberry Pi Pico 2](examples/rp235x/).
