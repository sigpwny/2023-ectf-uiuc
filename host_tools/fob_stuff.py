from fastecdsa import keys, curve
priv_key, pub_key = keys.gen_keypair(curve.P256)


file = open('EEPROM', "r+b") 
num_bytes = 2048
file.write(bytes("f", 'utf-8') * num_bytes)
offset = 0x100
file.seek(int(offset, base=16)) 
data = "something"
file.write(bytes(data, 'utf-8'))
priv_key, pub_key = keys.gen_keypair(curve.P256)

addresses = {
"FOBMEM_FOB_SECRET":    "0x100",
"FOBMEM_FOB_SECRET_ENC":"0x120",
"FOBMEM_FOB_SALT":      "0x140",
"FOBMEM_PIN_HASH":      "0x160",
"FOBMEM_CAR_ID":        "0x200", 
"FOBMEM_FEAT_1":        "0x204",
"FOBMEM_FEAT_2":        "0x208",
"FOBMEM_FEAT_3":        "0x20C",
"FOBMEM_FEAT_1_SIG":    "0x240",
"FOBMEM_FEAT_2_SIG":    "0x280",
"FOBMEM_FEAT_3_SIG":    "0x2C0",
"FOBMEM_CAR_PUBLIC":    "0x300",
"FOBMEM_FOB_IS_PAIRED": "0x400",
"FOBMEM_MSG_FEAT_3":    "0x700",
"FOBMEM_MSG_FEAT_2":    "0x740",
"FOBMEM_MSG_FEAT_1":    "0x780",
"FOBMEM_MSG_UNLOCK":    "0x7C0"
}

file = open('EEPROM', "r+b") 
num_bytes = 2048
file.write(bytes("f", 'utf-8') * num_bytes)
for key in addresses:
    offset = addresses[key]
    file.seek(int(offset, base=16)) 
    data = key
    file.write(bytes(data, 'utf-8')) 

file.close() 
file.close() 