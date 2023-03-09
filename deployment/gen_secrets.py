#!/usr/bin/env python3

import sys
from fastecdsa import keys, curve, ecdsa

"""
Generate two key pairs, one for the car and the fob, the other for the car and
the factory. Write each key pair to a file.
"""

secrets_dir = sys.argv[1]

# Generate key pairs
fob_sec, fob_pub = keys.gen_keypair(curve.P256)
car_sec, car_pub = keys.gen_keypair(curve.P256)
car_factory_sec, car_factory_pub = keys.gen_keypair(curve.P256)



# Write keys to files
with open(secrets_dir + '/fob_sec', 'wb+') as f:
    f.write(fob_sec.to_bytes(32, 'big'))

with open(secrets_dir + '/fob_pub', 'wb+') as f:
    f.write(fob_pub.x.to_bytes(32, 'big'))
    f.write(fob_pub.y.to_bytes(32, 'big'))

with open(secrets_dir + '/car_sec', 'wb+') as f:
    f.write(car_sec.to_bytes(32, 'big'))

with open(secrets_dir + '/car_pub', 'wb+') as f:
    f.write(car_pub.x.to_bytes(32, 'big'))
    f.write(car_pub.y.to_bytes(32, 'big'))

with open(secrets_dir + '/man_sec', 'wb+') as f:
    f.write(car_factory_sec.to_bytes(32, 'big'))

with open(secrets_dir + '/man_pub', 'wb+') as f:
    f.write(car_factory_pub.x.to_bytes(32, 'big'))
    f.write(car_factory_pub.y.to_bytes(32, 'big'))
