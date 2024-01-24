import serial
import argparse
import time
import crcmod.predefined

"""
How to use serial module besides python scrypt:
    python -m serial.tools.list_ports -v
    python -m serial.tools.miniterm <com_port> <baudrate>
"""

NL = "\n"
TAB = " " * 4
SEP = "-" * 80

FRAME_START_PATTERN = 0xAA
MINIMUM_FRAME_SIZE = 6
SIZE_OFFSET = 2
MESSAGE_OFFSET = 4


crc16 = crcmod.predefined.Crc('crc-16-usb')


class SerialLink:
    def __init__(self, port, baudrate) -> None:
        
        self.serial = serial.Serial(port, baudrate)

        print("Started serial link:")
        print(f"{TAB}Port:     {port}")
        print(f"{TAB}Baudrate: {baudrate}")
        print(f"{SEP}")


    def send_frame(self, message):
        self.serial.write(message)

    def listen_frame(self):
        buffer = bytearray()
        valid_frame = False

        while True:
            if self.serial.in_waiting > 0:

                #Read from serial
                #TODO decide how to proceed in the future. Actual code expects bytes, commented code is for strings
                byte1 = self.serial.read()
                # byte1_hex_str = byte1.decode('ascii')
                # byte2 = self.serial.read()
                # byte2_hex_str = byte2.decode('ascii')
                # pair = byte1_hex_str + byte2_hex_str
                # buffer.append(int(pair, 16))

                #Append data to the buffer
                buffer.append(int.from_bytes(byte1, byteorder='big'))


                while len(buffer) >= MINIMUM_FRAME_SIZE:

                    # Search for the frame start
                    if buffer[0] == FRAME_START_PATTERN and buffer[1] == FRAME_START_PATTERN:

                        #Extract the length of the data from the frame
                        data_len = int.from_bytes(buffer[SIZE_OFFSET:MESSAGE_OFFSET], byteorder='big')
                        
                        if len(buffer) >= data_len + MINIMUM_FRAME_SIZE:
                            
                            frame_data = buffer[MESSAGE_OFFSET:-2]
                            frame_crc = int.from_bytes(buffer[-2:], byteorder='big')

                            #Calculate CRC (calculated on size and data field from the frame)
                            crc16.update(buffer[SIZE_OFFSET:-2])
                            checksum = crc16.crcValue
                            
                            if frame_crc == checksum:
                                print(f"Received frame:\nLength: {data_len}\nUseful data: {frame_data.hex()}\nCRC: {hex(frame_crc)}\n\n")
                                valid_frame = True
                                break
                            else:
                                print(f"Received wrong CRC:\nLength: {data_len}\nUseful data: {frame_data.hex()}\nFrame CRC: {hex(frame_crc)}\nCRC: {hex(checksum)}\n\n\n")
                                
                            
                            # Remove this frame from the buffer
                            del buffer[:6 + data_len]
                        else:
                            break
                    else:
                        # Remove first byte to re-align frame search
                        del buffer[0]
            
                
                

            # Optional: add a small delay to prevent high CPU usage in the loop
            time.sleep(0.1)
            if valid_frame == True:
                break

    def close_serial(self):
        self.serial.close()

