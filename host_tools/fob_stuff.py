from fastecdsa import keys, curve
priv_key, pub_key = keys.gen_keypair(curve.P256)



file = open('EEPROM', "r+b") 
file.seek(2) 
offset = "something" 
file.write(bytes(offset, 'utf-8')) 
file.close() # ???