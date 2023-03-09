#!/usr/bin/env python3

import sys
import os
import hashlib

from fastecdsa import ecdsa, curve


secrets_dir = sys.argv[1]
eeprom_file = sys.argv[2]

# put the salted pin in eeprom if necessary
fob_sec = open(os.path.join(secrets_dir, "fob_sec"), "rb").read()
car_factory_sec = open(os.path.join(secrets_dir, "feat1_sig"), "rb").read()
feat1 = os.urandom(4)
feat2 = os.urandom(4)
feat3 = os.urandom(4)



if len(sys.argv) > 4:
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

    feat1_sig_a, feat1_sig_b = ecdsa.sign(car_id+feat1, int.from_bytes(car_factory_sec, "big"), curve.P256)
    feat1_sig = (feat1_sig_a+feat1_sig_b).to_bytes(64, "big")
    feat2_sig_a, feat2_sig_b = ecdsa.sign(car_id+feat2, int.from_bytes(car_factory_sec, "big"), curve.P256)
    feat2_sig = (feat2_sig_a+feat2_sig_b).to_bytes(64, "big")
    feat3_sig_a, feat3_sig_b = ecdsa.sign(car_id+feat3, int.from_bytes(car_factory_sec, "big"), curve.P256)
    feat3_sig = (feat3_sig_a+feat3_sig_b).to_bytes(64, "big")


else:
    car_id = None
    hashed_pin = None
    salt = None
    fob_sec_enc = None
    feat1_sig = None
    feat2_sig = None
    feat3_sig = None



addresses = {
    "FOBMEM_FOB_SECRET":     [0x100, fob_sec],
    "FOBMEM_FOB_SECRET_ENC": [0x120, fob_sec_enc],
    "FOBMEM_FOB_SALT":       [0x140, salt],
    "FOBMEM_PIN_HASH":       [0x160, hashed_pin],
    "FOBMEM_CAR_ID":         [0x200, car_id],
    "FOBMEM_FEAT_1":         [0x204, feat1],
    "FOBMEM_FEAT_2":         [0x208, feat2],
    "FOBMEM_FEAT_3":         [0x20C, feat3],
    "FOBMEM_FEAT_1_SIG":     [0x240, feat1_sig],
    "FOBMEM_FEAT_2_SIG":     [0x280, feat2_sig],
    "FOBMEM_FEAT_3_SIG":     [0x2C0, feat3_sig],
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
