import serial

ser = serial.Serial('COM5', 115200, timeout=0)

ser.write(b"\x40\x12\x34\x56")