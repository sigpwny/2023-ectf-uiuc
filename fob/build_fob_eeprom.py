#!/usr/bin/env python3

import sys
import os
import hashlib

from fastecdsa import ecdsa, curve


secrets_dir = sys.argv[1]
eeprom_file = sys.argv[2]

fob_secret = None
fob_secret_enc = None
fob_salt = os.urandom(12) # Generate fob-unique FOB_SALT
pin_hash = None
car_id = None
car_public = None
fob_is_paired = b"\x00\x00\x00\x00"

# If we are configuring a paired fob...
if len(sys.argv) > 4:
    # Prepare inputs as bytes
    fob_secret = open(os.path.join(secrets_dir, "fob_sec"), "rb").read()
    car_public = open(os.path.join(secrets_dir, "car_pub"), "rb").read()
    car_id = int(sys.argv[3]).to_bytes(4, "big")
    pair_pin = bytes.fromhex(sys.argv[4])

    # Generate PIN_HASH
    m = hashlib.sha256()
    m.update(fob_salt + b"\x00" + pair_pin)
    pin_hash = m.digest()

    # Generate FOB_SECRET_ENC
    m = hashlib.sha256()
    m.update(pair_pin + b"\x00" + fob_salt)
    fob_secret_enc = m.digest()
    fob_secret_enc = bytes(a ^ b for a, b in zip(fob_secret_enc, fob_secret))

    # Set fob to paired
    fob_is_paired = b"\x00\x00\x00\x01"

addresses = {
    "FOBMEM_FOB_SECRET":     [0x100, fob_secret],
    "FOBMEM_FOB_SECRET_ENC": [0x120, fob_secret_enc],
    "FOBMEM_FOB_SALT":       [0x140, fob_salt],
    "FOBMEM_PIN_HASH":       [0x160, pin_hash],
    "FOBMEM_CAR_ID":         [0x200, car_id],
    "FOBMEM_FEAT_1_SIG":     [0x240, None],
    "FOBMEM_FEAT_2_SIG":     [0x280, None],
    "FOBMEM_FEAT_3_SIG":     [0x2C0, None],
    "FOBMEM_CAR_PUBLIC":     [0x300, car_public],
    "FOBMEM_FOB_IS_PAIRED":  [0x400, fob_is_paired],
    "FOBMEM_MSG_FEAT_3":     [0x700, None],
    "FOBMEM_MSG_FEAT_2":     [0x740, None],
    "FOBMEM_MSG_FEAT_1":     [0x780, None],
    "FOBMEM_MSG_UNLOCK":     [0x7C0, None]
}

with open(eeprom_file, 'wb+') as f:
    # Write 0x00 to all bytes (from 0 to 2048)
    f.write(b'\xff' * 2048)

    for key, value in addresses.items():
        f.seek(value[0])
        if value[1] is not None:
            f.write(value[1])
