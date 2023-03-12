#!/usr/bin/env python3

import sys
import os
import hashlib

from fastecdsa import ecdsa, curve

if len(sys.argv) <= 3:
    print("Usage: build_car_eeprom.py <secrets_dir> <eeprom_file> <car_id>")
    sys.exit(1)

secrets_dir = sys.argv[1]
eeprom_file = sys.argv[2]

# Prepare inputs as bytes
car_secret = open(os.path.join(secrets_dir, "car_sec"), "rb").read()
man_public = open(os.path.join(secrets_dir, "man_pub"), "rb").read()
fob_public = open(os.path.join(secrets_dir, "fob_pub"), "rb").read()
car_id = int(sys.argv[3]).to_bytes(4, "big")

addresses = {
    "CARMEM_CAR_SECRET":     [0x100, car_secret],
    "CARMEM_MAN_PUBLIC":     [0x120, man_public],
    "CARMEM_FOB_PUBLIC":     [0x160, fob_public],
    "CARMEM_CAR_ID":         [0x200, car_id],
    "CARMEM_MSG_FEAT_3":     [0x700, None],
    "CARMEM_MSG_FEAT_2":     [0x740, None],
    "CARMEM_MSG_FEAT_1":     [0x780, None],
    "CARMEM_MSG_UNLOCK":     [0x7C0, None]
}

with open(eeprom_file, 'wb+') as f:
    # Initialize EEPROM with 0xFF
    f.write(b'\xff' * 2048)

    for key, value in addresses.items():
        f.seek(value[0])
        if value[1] is not None:
            f.write(value[1])
