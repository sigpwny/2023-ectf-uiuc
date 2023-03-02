#!/usr/bin/env python3

import sys
from pathlib import Path


BLOCK_SIZE = 16
PAGE_SIZE = 1024

FLASH_PAGES = 256
FLASH_SIZE = FLASH_PAGES * PAGE_SIZE
EEPROM_PAGES = 2
EEPROM_SIZE = EEPROM_PAGES * PAGE_SIZE

FW_FLASH_PAGES = 110
FW_FLASH_SIZE = FW_FLASH_PAGES * PAGE_SIZE

FW_EEPROM_PAGES = 2
FW_EEPROM_SIZE = FW_EEPROM_PAGES * PAGE_SIZE

def package_device(
    bin_path,
    eeprom_path,
    image_path,
):
    """
    Package a device image for use with the bootstrapper

    Accepts up to 64 bytes (encoded in hex) to insert as a secret in EEPROM
    """
    # Read input bin file
    bin_data = bin_path.read_bytes()

    # Pad bin data to max size
    image_bin_data = bin_data.ljust(FW_FLASH_SIZE, b"\xff")

    # Read EEPROM data
    if eeprom_path is not None:
        eeprom_data = eeprom_path.read_bytes()
    else:
        eeprom_data = b""

    # Pad EEPROM to max size
    image_eeprom_data = eeprom_data.ljust(FW_EEPROM_SIZE, b"\xff")

    # Create phys_image.bin
    image_data = image_bin_data + image_eeprom_data

    # Write output binary
    image_path.write_bytes(image_data)

def main():
    # this program is called with 3 arguments:
    # 1. the path to the output file
    # 2. the path to the input bin file
    # 3. (optional) the path to the input eeprom file
    # it will create a file at the output path with the inputs padded and
    # concatenated

    # parse arguments
    output_path = Path(sys.argv[1])
    bin_path = Path(sys.argv[2])
    if len(sys.argv) > 3:
        eeprom_path = Path(sys.argv[3])
    else:
        eeprom_path = None

    # package the device
    package_device(bin_path, eeprom_path, output_path)

if __name__ == "__main__":
    main()