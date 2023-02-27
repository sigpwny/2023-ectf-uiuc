# State

## Secrets and Variables

- Fob keypair
  - `FOB_SECRET` - 256 bits (32 bytes), P-256 private key
  - `FOB_PUBLIC` - 256 bits (32 bytes), P-256 public key
- Car keypair
  - `CAR_SECRET` - 256 bits (32 bytes), P-256 private key
  - `CAR_PUBLIC` - 256 bits (32 bytes), P-256 public key
- Manufacturer keypair
  - `MAN_SECRET` - 256 bits (32 bytes), P-256 private key
  - `MAN_PUBLIC` - 256 bits (32 bytes), P-256 public key

Data:
- `FEAT_1` - 8 bits (1 byte)
- `FEAT_1_SIG` - 512 bits (64 bytes), P-256 signature from factory
- `FEAT_2` - 8 bits (1 byte)
- `FEAT_2_SIG` - 512 bits (64 bytes), P-256 signature from factory
- `FEAT_3` - 8 bits (1 byte)
- `FEAT_3_SIG` - 512 bits (64 bytes), P-256 signature from factory

Pairing-specific state:
- `FOB_SECRET_ENC` - 256 bits (32 bytes), AES-128-CBC encrypted data
- `FOB_SALT` - 96 bits (12 bytes)
  - Combined with unhashed, big-endian PIN (3 bytes and 1 byte padding): `FOB_SALT + (0x00 + PIN) => 16 bytes`
  - Used as a salt to validate password against stored hash and also used to decrypt `FOB_SECRET_ENC`
- `PIN_HASH` - ???
- `FOB_IS_PAIRED` - 1 byte

## EEPOM State

