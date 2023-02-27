# State

## Secrets and Variables

- Fob keypair
  - `FOB_SECRET` - 256 bits (32 bytes)
  - `FOB_PUBLIC` - 256 bits (32 bytes)
- Car keypair
  - `CAR_SECRET` - 256 bits (32 bytes)
  - `CAR_PUBLIC` - 256 bits (32 bytes)
- Manufacturer keypair
  - `MAN_SECRET` - 256 bits (32 bytes)
  - `MAN_PUBLIC` - 256 bits (32 bytes)

Data:
- `FEAT_1`

Pairing-specific state:
- `FOB_SECRET_ENC` - 256 bits (32 bytes), AES-128-CBC encrypted fob secret
- `FOB_SALT` - 96 bits (12 bytes)
  - Combined with unhashed, big-endian PIN (3 bytes and 1 byte padding): `FOB_SALT + (0x00 + PIN) => 16 bytes`
  - Used as a salt to validate password against stored hash and also used to decrypt `FOB_SECRET_ENC`
- `PIN_HASH` - ???
- `FOB_IS_PAIRED` - 1 byte

## EEPOM State

