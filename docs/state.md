# State

## Secrets and Variables

All are unsigned unless otherwise specified

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
- `CAR_ID` - 8 bits (1 byte), stored as 32 bits (4 bytes)
- `FEAT_1` - 8 bits (1 byte), stored as 32 bits (4 bytes)
- `FEAT_1_SIG` - 512 bits (64 bytes), P-256 signature from factory
- `FEAT_2` - 8 bits (1 byte), stored as 32 bits (4 bytes)
- `FEAT_2_SIG` - 512 bits (64 bytes), P-256 signature from factory
- `FEAT_3` - 8 bits (1 byte), stored as 32 bits (4 bytes)
- `FEAT_3_SIG` - 512 bits (64 bytes), P-256 signature from factory

Pairing-specific state:
- `FOB_SECRET_ENC` - 256 bits (32 bytes), AES-128-CBC encrypted data
- `FOB_SALT` - 96 bits (12 bytes)
  - Combined with unhashed, big-endian PIN (3 bytes and 1 byte padding): `FOB_SALT + (0x00 + PIN) => 16 bytes`
  - Used as a salt to validate password against stored hash and also used to decrypt `FOB_SECRET_ENC`
- `PIN_HASH` - ???
- `FOB_IS_PAIRED` - 32 bits (4 bytes)

## EEPOM State

### Car EEPROM
```
0x000┌─────────────────────┬───┐
     │                     │-  │
0x100├─────────────────────┼───┤
     │CAR_SECRET           │R  │
0x120├─────────────────────┼───┤
     │MAN_PUBLIC           │R  │
0x140├─────────────────────┼───┤
     │FOB_PUBLIC           │R  │
0x160├─────────────────────┼───┤
     │CAR_ID               │R  │
0x164├─────────────────────┼───┤
     │                     │-  │
     │                     │   │
     │                     │   │
     │                     │   │
     │                     │   │
0x700├─────────────────────┼───┤ <-- End of allowed PARED EEPROM
     │Feature 3 Message    │R  │
0x740├─────────────────────┼───┤
     │Feature 2 Message    │R  │
0x780├─────────────────────┼───┤
     │Feature 1 Message    │R  │
0x7C0├─────────────────────┼───┤
     │Unlock Message       │R  │
0x800└─────────────────────┴───┘
```

### Fob EEPROM
```
0x000┌─────────────────────┬───┐
     │                     │-  │
0x100├─────────────────────┼───┤
     │FOB_SECRET           │RW │
0x120├─────────────────────┼───┤
     │FOB_SECRET_ENC       │RW │
0x140├─────────────────────┼───┤
     │FOB_SALT             │R  │
0x14C├─────────────────────┼───┤
     │                     │-  │
0x160├─────────────────────┼───┤
     │PIN_HASH             │RW │
  ???├─────────────────────┼───┤
     │                     │-  │
0x200├─────────────────────┼───┤
     │FOB_IS_PAIRED        │RW │
0x204├─────────────────────┼───┤
     │FEAT_1               │RW │
0x208├─────────────────────┼───┤
     │FEAT_2               │RW │
0x20C├─────────────────────┼───┤
     │FEAT_3               │RW │
0x210├─────────────────────┼───┤
     │                     │-  │
0x240├─────────────────────┼───┤
     │FEAT_1_SIG           │RW │
0x280├─────────────────────┼───┤
     │FEAT_2_SIG           │RW │
0x2C0├─────────────────────┼───┤
     │FEAT_3_SIG           │RW │
0x300├─────────────────────┼───┤
     │CAR_ID (unused)      │RW │
0x301├─────────────────────┼───┤
     │                     │-  │
0x320├─────────────────────┼───┤
     │CAR_PUBLIC           │RW │
0x340├─────────────────────┼───┤
     │                     │-  │
0x700├─────────────────────┼───┤ <-- End of allowed PARED EEPROM
     │Feature 3 Message    │R  │
0x740├─────────────────────┼───┤
     │Feature 2 Message    │R  │
0x780├─────────────────────┼───┤
     │Feature 1 Message    │R  │
0x7C0├─────────────────────┼───┤
     │Unlock Message       │R  │
0x800└─────────────────────┴───┘
```
