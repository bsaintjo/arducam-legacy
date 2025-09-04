# Example of image capture on STM32U0

This example has been tested on a STM32U083RC. It captures an image from a Arducam 2MP Mini attached directly to the board.

Must have `probe-rs` installed for flashing.

```bash
cargo run --release
```

As a quick hack to look at the image generated, I convert the image bytes into JSON using serde and print it to stdout. You need to disable other logging some and delete the last line.

```bash
DEFMT_LOG=off cargo run --release | head -n 1 >jpeg.json
```

You can use the included `convert.py` to convert the JSON file into a JPEG you can view normally.

```bash
$ cat jpeg.json | python3 convert.py
JPEG file created successfully: output.jpg
```

This can also be piped together to achieve an incredibly slow video camera feed.

```bash
$ DEFMT_LOG=off cargo run --release | python3 convert.py
JPEG file created successfully: output.jpg
JPEG file created successfully: output.jpg
# Open output.jpg and see file updates.
```
