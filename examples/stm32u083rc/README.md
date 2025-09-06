# Example of image capture on STM32U0

This example has been tested on a STM32U083RC. It captures an image from a Arducam 2MP Mini attached directly to the board.

Must have `probe-rs` installed for flashing.

```bash
cargo run --release
```

As a quick hack to look at the image generated, I convert the image bytes into JSON using serde and print it to stdout. You need to disable other logging some and delete the last line.

```bash
cargo run --release | head -n 1 >jpeg.json
```

You can use the included `convert.py` to convert the JSON file into a JPEG you can view normally.

```bash
$ cat jpeg.json | python3 convert.py
JPEG file created successfully: output.jpg
```

This can also be piped together to achieve an incredibly slow video camera feed.

```bash
$ cargo run --release | python3 convert.py
JPEG file created successfully: output.jpg
JPEG file created successfully: output.jpg
# Open output.jpg and see file updates.
```

## Known problems

Currently the async/DMA version doesn't seem to work properly on the STM32083RC. I had to switch to a different board, and a working version can be found in the [rp235x](../examples/rp235x/src/bin/arducam-async.rs) example. In particular, I2C async works but there is some issue with the SPI Async/DMA interface.

If you are trying to adapt this example to another STM32 board, try to run the [spi_dma](src/bin/spi_dma.rs) example, and if that works, then there might be a chance the arducam async examples will too.
