#!/usr/bin/env python3

import sys
from fastecdsa import keys, curve, ecdsa

"""
Generate three key pairs: one for the fob, one for the car, and one for the 
manufacturer.
"""

secrets_dir = sys.argv[1]

# Generate key pairs
fob_sec, fob_pub = keys.gen_keypair(curve.P256)
car_sec, car_pub = keys.gen_keypair(curve.P256)
man_sec, man_pub = keys.gen_keypair(curve.P256)

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
    f.write(man_sec.to_bytes(32, 'big'))

with open(secrets_dir + '/man_pub', 'wb+') as f:
    f.write(man_pub.x.to_bytes(32, 'big'))
    f.write(man_pub.y.to_bytes(32, 'big'))
