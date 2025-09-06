#!/usr/bin/env python3
"""
This script reads JSON from stdin, with a bytes keys that contains bytes for a JPEG file and writes them to a file named output.jpg

This is a simple utility for testing output from the driver by reading the transferred bytes from the probe-rs without setting up an entire USB
setup.
An example of using this with one of the examples:

cargo run --release --bin blocking | python3 convert.py
"""

import json


def main():
    while True:
        try:
            json_file = input()
            data = json.loads(json_file)

            byte_data = bytes(data["bytes"])

            with open("output.jpg", "wb") as img_file:
                img_file.write(byte_data)

            print("JPEG file created successfully: output.jpg")

        except json.JSONDecodeError:
            # Ignore log lines so we can run this without turning off logging
            print("Skipping log line...")
            continue

        except (EOFError, KeyboardInterrupt):
            print("Stopping...")
            break


if __name__ == "__main__":
    main()
