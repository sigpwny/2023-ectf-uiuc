# PwnyPARED Protocol

> **Warning**  
> This protocol is still under active development and changes can be made at any point.

The PwnyPARED protocol dictates UART communication for SIGPwny's implementation of a car and keyfob system for eCTF 2023.

## Building (Host Tools)

## Pairing Fobs

**PAIR_REQ**

**PAIR_SYN**

**PAIR_SYN_ACK**

**PAIR_WRITE**

```mermaid
sequenceDiagram
  participant Host Computer
  participant Paired Fob
  participant Unpaired Fob
  Host Computer->>Paired Fob: PAIR_REQ <br/>PIN: 123456
  Paired Fob->>Unpaired Fob: PAIR_SYN <br/>PIN: 123456
  alt No PAIR_SYN_ACK
    Paired Fob-xHost Computer: "Paired fob: Could not find unpaired fob"
  else Got PAIR_SYN_ACK
    Unpaired Fob-->>Paired Fob: PAIR_SYN_ACK
    alt PIN incorrect
      Paired Fob-xHost Computer: (After maximum 5s delay) <br/>"Paired fob: PIN is incorrect"
    else PIN correct
      Paired Fob->>Unpaired Fob: PAIR_WRITE <br/>(secrets)
    end
  end
  Note right of Unpaired Fob: Unpaired fob <br/>now paired!
  Unpaired Fob->>Host Computer: "Unpaired fob: Successfully paired!
```

## Packaging Features

## Enabling Features

## Unlocking Car
```mermaid
sequenceDiagram
    participant Host Computer
    participant Car
    participant Fob
    Fob->>Car: Unlock request
    alt No_Unsigned_Nonce
        Car ->> Host Computer: "Unlock failed: No return of unsigned nonce."
    end
    Car->>Fob: Unsigned nonce
    alt No_Signed_Nonce
        Car ->> Host Computer: "Unlock failed: No return of signed nonce."
    end
        Fob->>Car: Signed nonce
        Car->>Host Computer: Unlock succeeded
    Note right of Car: Car unlocked
```
