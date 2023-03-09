#!/usr/bin/env python3

import sys
import os
import hashlib

from fastecdsa import ecdsa, curve


secrets_dir = sys.argv[1]
eeprom_file = sys.argv[2]

# put the salted pin in eeprom if necessary

if len(sys.argv) > 4:
    fob_sec = open(os.path.join(secrets_dir, "fob_sec"), "rb").read()
    car_id = int(sys.argv[3], base=16).to_bytes(4, "big")
    pair_pin = bytes.fromhex(sys.argv[4])

    # generate salt on the fly
    salt = os.urandom(12)
    m = hashlib.sha256()
    m.update(salt + b"\x00" + pair_pin)
    hashed_pin = m.digest()

    m = hashlib.sha256()
    m.update(pair_pin + b"\x00" + salt)
    fob_sec_enc = m.digest()
    fob_sec_enc = bytes(a ^ b for a, b in zip(fob_sec_enc, fob_sec))


else:
    fob_sec = None
    fob_sec_enc = None
    car_id = None
    hashed_pin = None
    salt = None

addresses = {
    "FOBMEM_FOB_SECRET":     [0x100, fob_sec],
    "FOBMEM_FOB_SECRET_ENC": [0x120, fob_sec_enc],
    "FOBMEM_FOB_SALT":       [0x140, salt],
    "FOBMEM_PIN_HASH":       [0x160, hashed_pin],
    "FOBMEM_CAR_ID":         [0x200, car_id],
    "FOBMEM_FEAT_1":         [0x204, None],
    "FOBMEM_FEAT_2":         [0x208, None],
    "FOBMEM_FEAT_3":         [0x20C, None],
    "FOBMEM_FEAT_1_SIG":     [0x240, None],
    "FOBMEM_FEAT_2_SIG":     [0x280, None],
    "FOBMEM_FEAT_3_SIG":     [0x2C0, None],
    "FOBMEM_CAR_PUBLIC":     [0x300, open(os.path.join(secrets_dir, "car_pub"), "rb").read()],
    "FOBMEM_FOB_IS_PAIRED":  [0x400, None],
    "FOBMEM_MSG_FEAT_3":     [0x700, None],
    "FOBMEM_MSG_FEAT_2":     [0x740, None],
    "FOBMEM_MSG_FEAT_1":     [0x780, None],
    "FOBMEM_MSG_UNLOCK":     [0x7C0, None]
}

with open(eeprom_file, 'wb+') as f:
    # write 0x00 to all addresses (from 0 to 2048)
    f.write(b'\x00' * 2048)

    for key, value in addresses.items():
        f.seek(value[0])
        if value[1] is not None:
            f.write(value[1])
