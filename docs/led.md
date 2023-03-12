# LEDs

## Unlocking
Fob:
- If the blue LED turns on once, then the fob was able to receive the nonce from the car. If the blue LED turns off and the red LED turns on, then the fob was unable to verify the signature from the car's nonce. If the blue LED turns off and the red LED stays off, then the fob was able to verify the signature from the car's nonce and has sent an UNLOCK_RESP.
- If the green LED turns on, then the fob received UNLOCK_GOOD from the car. Once it turns off, the fob has sent UNLOCK_FEAT.

Car:
- If the blue LED turns on, then the car has received an UNLOCK_REQ and is in the process of sending the nonce to the fob. If the blue LED turns off and the red LED turns on, then the car was unable to verify the signature from the fob's nonce. If the blue LED turns off, the red LED stays off, and the green LED turns on, then the car was able to verify the signature from the fob's nonce and is about to send an UNLOCK_GOOD. If the blue LED turns back on, then the car has received UNLOCK_FEAT and is now verifying and sending the feature messages. Once all LEDs turn off, the car has finished the unlock process.

- Blue LED on car: Car is in process of unlocking.
- Blue LED on fob: Fob received nonce from car and is verifying nonce.
- Red LED on fob, no LED on car: Fob could not verify signature from car's nonce.
