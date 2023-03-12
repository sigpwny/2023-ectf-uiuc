# State

State used in our implementation of PwnyPARED. All values are 
unsigned unless otherwise specified.

## Secrets
### Fob keypair
- `FOB_SECRET` - 32 bytes, P-256 private key
- `FOB_PUBLIC` - 64 bytes, P-256 public key
### Car keypair
- `CAR_SECRET` - 32 bytes, P-256 private key
- `CAR_PUBLIC` - 64 bytes, P-256 public key
### Manufacturer keypair
- `MAN_SECRET` - 32 bytes, P-256 private key
- `MAN_PUBLIC` - 64 bytes, P-256 public key

## Variables
### General state
- `CAR_ID` - 4 bytes
- `FEAT_NUM` - 4 bytes, can only be values 1, 2, or 3
- `FEAT_1_SIG` - 64 bytes, P-256 signature from manufacturer
- `FEAT_2_SIG` - 64 bytes, P-256 signature from manufacturer
- `FEAT_3_SIG` - 64 bytes, P-256 signature from manufacturer

### Pairing-specific state
- `PIN` - 3 bytes, PIN entered by user (padded to 4 bytes with a null byte for 
hashes)
- `FOB_SALT` - 12 bytes, a secret which is unique to each fob
  - Used as a salt to validate password against stored hash and also used to 
  decrypt `FOB_SECRET_ENC`
- `FOB_SECRET_ENC` - 32 bytes, copy of `FOB_SECRET` XOR'd with SHA256 hash of 
`PIN` and `FOB_SALT`
- `PIN_HASH` - 32 bytes, SHA256 hash of `FOB_SALT` and `PIN` used to validate 
PIN
- `FOB_IS_PAIRED` - 4 bytes, 1 if fob is paired, 0 if unpaired

### Unlocking-specific state
- `NONCE` - 8 bytes, random number used to prevent replay attacks
- `NONCE_SIG` - 64 bytes, P-256 signature of `NONCE` from car or fob 

## EEPOM

### Car EEPROM
```
0x000┌─────────────────────┬───┐
     │                     │-  │
0x100├─────────────────────┼───┤
     │CAR_SECRET           │R  │
0x120├─────────────────────┼───┤
     │MAN_PUBLIC           │R  │
0x160├─────────────────────┼───┤
     │FOB_PUBLIC           │R  │
0x1A0├─────────────────────┼───┤
     │                     │-  │
0x200├─────────────────────┼───┤
     │CAR_ID               │R  │
0x204├─────────────────────┼───┤
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
0x180├─────────────────────┼───┤
     │                     │-  │
0x200├─────────────────────┼───┤
     │CAR_ID (unused)      │RW │
0x204├─────────────────────┼───┤
     │                     │-  │
0x240├─────────────────────┼───┤
     │FEAT_1_SIG           │RW │
0x280├─────────────────────┼───┤
     │FEAT_2_SIG           │RW │
0x2C0├─────────────────────┼───┤
     │FEAT_3_SIG           │RW │
0x300├─────────────────────┼───┤
     │CAR_PUBLIC           │RW │
0x340├─────────────────────┼───┤
     │                     │-  │
0x400├─────────────────────┼───┤
     │FOB_IS_PAIRED        │RW │
0x404├─────────────────────┼───┤
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
