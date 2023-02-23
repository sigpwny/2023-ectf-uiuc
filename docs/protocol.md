# PwnyPARED Protocol

> **Warning**  
> This protocol is still under active development and changes can be made at 
any point.

The PwnyPARED protocol dictates UART communication for SIGPwny's 
implementation of a car and keyfob system for eCTF 2023.

> **Note**  
> "TTT" refers to "total transaction time."

## Building (Host Tools)

*TODO*

## Pairing Fobs

```mermaid
sequenceDiagram
  participant Host Computer
  participant Paired Fob
  participant Unpaired Fob
  Host Computer ->> Paired Fob: PAIR_REQ <br/>PIN: 0x123456
  Note over Paired Fob: PIN is validated, <br/>result is stored
  Paired Fob ->> Unpaired Fob: PAIR_SYN <br/>PIN: 0x123456
  alt No PAIR_ACK
    Paired Fob -x Host Computer: "Paired fob: Could not find unpaired fob"
  end
  Unpaired Fob -->> Paired Fob: PAIR_ACK
  Note over Host Computer, Unpaired Fob: Minimum 0.5s TTT elapsed
  alt PIN incorrect
    Paired Fob -x Host Computer: "Paired fob: PIN is incorrect"
    Paired Fob -x Unpaired Fob: PAIR_RST
    Note over Paired Fob: UART blocked until 5s TTT
  end
  Paired Fob ->> Host Computer: "Paired fob: PIN is correct"
  Paired Fob ->> Unpaired Fob: PAIR_FIN <br/>(Send secrets/features)
  alt PAIR_FIN takes too long
    Unpaired Fob -x Host Computer: "Unpaired fob: Fob data did not transfer in time"
    Note over Unpaired Fob: UART blocked until 5s TTT
  end
  Note over Unpaired Fob: Write secrets/features to <br/>EEPROM, fob is paired
  Unpaired Fob ->> Host Computer: "Unpaired fob: Successfully paired!"
  Note over Host Computer: <1s TTT on success
```

### PAIR_REQ
Sent by the host computer to initialize the paired fob for the pairing 
process. The paired fob checks the PIN (combined with the paired fob salt) 
against the hashed PIN stored in its EEPROM. Once done, it attempts to 
synchronize with the unpaired fob.

|             | Magic     | PIN               |
| ----------- | --------- | ----------------- |
| **Bytes**   | `\x40`    | `\x??\x??\x??`    |
| **Offsets** | 0x0 - 0x1 | 0x1 - 0x4         | 
| **Notes**   |           | Packed big-endian |

### PAIR_SYN
Sent by the paired fob to initialize the unpaired fob for the pairing process. 
The paired fob then waits for `PAIR_ACK`. If `PAIR_ACK` is not received after 
500ms TTT, an error is sent to the host computer.

When the unpaired fob receives `PAIR_SYN`, it will store the PIN in a 
variable, then send a `PAIR_ACK`.

|             | Magic     | PIN               |
| ----------- | --------- | ----------------- |
| **Bytes**   | `\x41`    | `\x??\x??\x??`    |
| **Offsets** | 0x0 - 0x1 | 0x1 - 0x4         | 
| **Notes**   |           | Packed big-endian |

### PAIR_ACK
Sent by the unpaired fob to the paired fob after it saves the PIN from 
`PAIR_SYN`. The paired fob will start decrypting the encrypted car secret with 
the provided PIN and stored salt, regardless if the PIN is correct or not. 
After this, if 500ms TTT has not yet elapsed, the paired fob will wait until 
then.

After this delay, if the PIN is incorrect, it will send an error to the host 
computer and a `PAIR_RST` to the unpaired fob. It will also block UART until 
5000ms TTT. If the PIN is correct, it will send a `PAIR_FIN` along with the 
fob data to the unpaired fob.

|             | Magic     |
| ----------- | --------- |
| **Bytes**   | `\x42`    |
| **Offsets** | 0x0 - 0x1 |
| **Notes**   |           |

### PAIR_FIN
Sent by the paired fob to the unpaired fob to transfer fob data. The 
transmitted fob data includes the decrypted car secret and three features. If 
there are less than three features, null bytes will be sent in place of 
missing features. This ensures that the payload is of fixed length.

If more than 500ms passes while the unpaired fob awaits the entire payload to 
be sent (over 1000ms TTT), then the unpaired fob will send an error message to 
the host computer and block UART for an additional 3500ms (<5000ms TTT).

Otherwise, once the entire payload is received, the unpaired fob will write 
and recreate the EEPROM structure of the paired fob (using its own salt to 
encrypt the car secret). A success message is sent to the host computer once 
this is completed.

*TODO: Update with correct secret and feature lengths*

|             | Magic     | Car Secret            | Feature 1   | Feature 2   | Feature 3   |
| ----------- | --------- | --------------------- | ----------- | ----------- | ----------- |
| **Bytes**   | `\x43`    |  \<xxx bytes\>        | \<yy> bytes | \<yy> bytes | \<yy> bytes |
| **Offsets** | 0x0 - 0x1 | 0x1 - 0x??            | 0x?? - 0x?? | 0x?? - 0x?? | 0x?? - 0x?? |
| **Notes**   |           | In order (big-endian) | Sent as-is  | Sent as-is  | Sent as-is  |

### PAIR_RST
If received, the fob will exit the current transaction (reset). The fob is not 
guaranteed to be listening for a reset.

|             | Magic     |
| ----------- | --------- |
| **Bytes**   | `\x44`    |
| **Offsets** | 0x0 - 0x1 |
| **Notes**   |           |

## Packaging Features

*TODO*

## Enabling Features

*TODO*

## Unlocking Car

```mermaid
sequenceDiagram
  participant Host Computer
  participant Car
  participant Fob
  Host Computer ->> Car: Start unlock (host)
  Fob ->> Car: Unlock request (SW1)
  Car ->> Host Computer: "Unlock requested"
  Car ->> Fob: Unsigned nonce
  alt No signed nonce
    Car ->> Host Computer: "Unlock failed: No signed nonce returned"
  end
  Fob->>Car: Signed nonce
  Note over Host Computer, Fob: Minimum 0.5s TTT elapsed
  alt Invalid signed nonce
    Car->>Host Computer: "Unlock failed: Invalid signed nonce"
    Note over Car: UART blocked for 5s TTT
  end
  Car->>Host Computer: "Unlock successful!" <br/>Print car message in EEPROM
  Note right of Car: Car unlocked
  Note over Host Computer, Fob: TODO: Transfer and print features
  Note over Host Computer: <1s TTT on success
```

*TODO*