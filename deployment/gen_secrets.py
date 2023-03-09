#!/usr/bin/env python3

import sys
from fastecdsa import keys, curve

"""
Generate two key pairs, one for the car and the fob, the other for the car and
the factory. Write each key pair to a file.
"""

secrets_dir = sys.argv[1]

# Generate key pairs
car_fob_priv, car_fob_pub = keys.gen_keypair(curve.P256)
car_factory_priv, car_factory_pub = keys.gen_keypair(curve.P256)

# Write keys to files
with open(secrets_dir + '/car_fob_priv', 'wb+') as f:
    f.write(car_fob_priv.to_bytes(32, 'big'))

with open(secrets_dir + '/car_fob_pub', 'wb+') as f:
    f.write(car_fob_pub.x.to_bytes(32, 'big'))
    f.write(car_fob_pub.y.to_bytes(32, 'big'))

with open(secrets_dir + '/car_factory_priv', 'wb+') as f:
    f.write(car_factory_priv.to_bytes(32, 'big'))

with open(secrets_dir + '/car_factory_pub', 'wb+') as f:
    f.write(car_factory_pub.x.to_bytes(32, 'big'))
    f.write(car_factory_pub.y.to_bytes(32, 'big'))
