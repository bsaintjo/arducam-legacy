#!/usr/bin/env python3
import json
import argparse


def main():
    # parser = argparse.ArgumentParser()
    # parser.add_argument("json_file")
    # args = parser.parse_args()
    # json_file = args.json_file
    # with open(json_file, "r") as f:
    #     data = json.load(f)
    while True:
        try:
            json_file = input()
            data = json.loads(json_file)


            byte_data = bytes(data["bytes"])

            with open("output.jpg", "wb") as img_file:
                img_file.write(byte_data)

            print("JPEG file created successfully: output.jpg")

        except (EOFError, KeyboardInterrupt):
            print("Stopping...")
            break


if __name__ == "__main__":
    main()
