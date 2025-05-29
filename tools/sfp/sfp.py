import serial # type: ignore
import struct
import argparse
import crcmod.predefined # type: ignore
import threading
import time
import sys

"""
How to use sfp module:
    1. create a pair of virtual serial ports -> socat -d -d pty,raw,echo=0 pty,raw,echo=0   
    2. terminal 1 - receiver -> python3 sfp.py -p /dev/ttys014 -b 115200 
    3. terminal 2 - sender -> python3 sfp.py -p /dev/ttys012 -b 115200 -i 
message: (normal text) hello or /sfp hello (sent as a sfp frame)

Usage examples:
    python3 sfp.py -p /dev/ttys012 -b 115200 -> sfp enabled
    python3 sfp.py -p /dev/ttys012 -b 115200 --no-sfp -> sfp disabled
    python3 sfp.py -p /dev/ttys012 -b 115200 -i -> input mode enabled
    python3 sfp.py -p /dev/ttys012 -b 115200 --no-sfp -i -> input mode enabled, SFP disabled
"""


NL = "\n"
TAB = " " * 4
SEP = "-" * 80


class SFP:
    FRAME_MARKER = bytes((0xAA, 0xAA))
    DATA_LEN_MIN = 1
    DATA_LEN_MAX = 1024
    VERSION = "0.1.0"
    
    def __init__(self, serial_obj=None):
        self.crc16 = crcmod.predefined.mkCrcFun('crc-ccitt-false')
        self.serial_port = serial_obj
        
    def calculate_crc(self, data_length, data):
        buffer = struct.pack('<H', data_length) + data
        return self.crc16(buffer)
    
    def send_frame(self, data):
        if not self.serial_port or not self.serial_port.is_open:
            raise ConnectionError("Serial port is not open")
        
        if not isinstance(data, bytes):
            data = bytes(data)
            
        data_length = len(data)
        if data_length < self.DATA_LEN_MIN or data_length > self.DATA_LEN_MAX:
            raise ValueError(f"Data length must be between {self.DATA_LEN_MIN} and {self.DATA_LEN_MAX} bytes")
        
        crc = self.calculate_crc(data_length, data)
        
        frame = (
            self.FRAME_MARKER +                 
            struct.pack('<H', data_length) +     
            data +                               
            struct.pack('<H', crc)               
        )
        
        bytes_sent = self.serial_port.write(frame)
        return bytes_sent
    
    def process_incoming_byte(self, byte, state_machine):
        state = state_machine['state']
        marker_byte_1 = self.FRAME_MARKER[0]
        marker_byte_2 = self.FRAME_MARKER[1]
        
        if state == 'WAIT_MARKER_1':
            if byte == marker_byte_1:
                return 'WAIT_MARKER_2', False, None
            return 'WAIT_MARKER_1', False, None
            
        elif state == 'WAIT_MARKER_2':
            if byte == marker_byte_2:
                state_machine['data_buffer'] = bytearray()
                state_machine['length_buffer'] = bytearray()
                return 'WAIT_LENGTH_1', False, None
            return 'WAIT_MARKER_1', False, None
            
        elif state == 'WAIT_LENGTH_1':
            state_machine['length_buffer'].append(byte)
            return 'WAIT_LENGTH_2', False, None
            
        elif state == 'WAIT_LENGTH_2':
            state_machine['length_buffer'].append(byte)
            data_length = struct.unpack('<H', state_machine['length_buffer'])[0]
            
            if data_length < self.DATA_LEN_MIN or data_length > self.DATA_LEN_MAX:
                return 'WAIT_MARKER_1', False, None
                
            state_machine['data_length'] = data_length
            state_machine['bytes_read'] = 0
            return 'WAIT_DATA', False, None
            
        elif state == 'WAIT_DATA':
            state_machine['data_buffer'].append(byte)
            state_machine['bytes_read'] += 1
            
            if state_machine['bytes_read'] >= state_machine['data_length']:
                state_machine['crc_buffer'] = bytearray()
                return 'WAIT_CRC_1', False, None
            return 'WAIT_DATA', False, None
            
        elif state == 'WAIT_CRC_1':
            state_machine['crc_buffer'].append(byte)
            return 'WAIT_CRC_2', False, None
            
        elif state == 'WAIT_CRC_2':
            state_machine['crc_buffer'].append(byte)
            received_crc = struct.unpack('<H', state_machine['crc_buffer'])[0]
            calculated_crc = self.calculate_crc(
                state_machine['data_length'], 
                state_machine['data_buffer']
            )
            
            if received_crc == calculated_crc:
                return 'WAIT_MARKER_1', True, bytes(state_machine['data_buffer'])
            return 'WAIT_MARKER_1', False, None
            
        return 'WAIT_MARKER_1', False, None


class SerialLinkSFP:
    def __init__(self, port, baudrate, enable_sfp=True, input_mode=False) -> None:
        self.enable_sfp = enable_sfp
        self.input_mode = input_mode
        self.running = False
        self.sfp_state_machine = {
            'state': 'WAIT_MARKER_1',
            'data_buffer': bytearray(),
            'length_buffer': bytearray(),
            'crc_buffer': bytearray(),
            'data_length': 0,
            'bytes_read': 0
        }
        
        try:
            self.serial_obj = serial.Serial()
            self.serial_obj.port = port
            self.serial_obj.baudrate = baudrate
            self.serial_obj.timeout = 0.1
            self.serial_obj.open()
            
            self.sfp = SFP(self.serial_obj)
            
            print("Started serial link:")
            print(f"{TAB}Port:     {self.serial_obj.port}")
            print(f"{TAB}Baudrate: {self.serial_obj.baudrate}")
            print(f"{TAB}SFP Mode: {'Enabled' if self.enable_sfp else 'Disabled'}")
            if self.input_mode:
                print(f"{TAB}Input Mode: Enabled - You can type messages to send")
            print(f"{SEP}")
            
            self.running = True
            
            self.receiver_thread = threading.Thread(target=self.receive_loop)
            self.receiver_thread.daemon = True
            self.receiver_thread.start()
            
            if self.input_mode:
                self.input_thread = threading.Thread(target=self.input_loop)
                self.input_thread.daemon = True
                self.input_thread.start()
            
            while self.running:
                time.sleep(0.1)
                
        except KeyboardInterrupt:
            self.running = False
            print(f"{SEP}{NL}User requested exit")
        except Exception as e:
            self.running = False
            print(f"{SEP}{NL}Error: {e}")
        finally:
            if hasattr(self, 'serial_obj') and self.serial_obj.is_open:
                self.serial_obj.close()
                print("Serial port closed")
    
    def receive_loop(self):
        try:
            while self.running and self.serial_obj.is_open:
                byte_data = self.serial_obj.read(1)
                
                if not byte_data: 
                    continue
                    
                if self.enable_sfp:
                    new_state, frame_complete, frame_data = self.sfp.process_incoming_byte(
                        byte_data[0], self.sfp_state_machine
                    )
                    self.sfp_state_machine['state'] = new_state
                    
                    if frame_complete and frame_data:
                        self.handle_sfp_frame(frame_data)
                try:
                    char = byte_data.decode('ascii')
                    if char.isprintable() or char in '\n\r\t':
                        sys.stdout.write(char)
                        sys.stdout.flush()
                except UnicodeDecodeError:
                    if not self.enable_sfp: 
                        hex_value = byte_data.hex().upper()
                        sys.stdout.write(f"[{hex_value}]")
                        sys.stdout.flush()
        
        except Exception as e:
            print(f"\nReceiver thread error: {e}")
            self.running = False
    
    def input_loop(self):
        try:
            while self.running and self.serial_obj.is_open:
                user_input = input()
                
                if not user_input:
                    continue
                
                if user_input.startswith("/sfp "):
                    data = user_input[5:].encode('utf-8')
                    try:
                        bytes_sent = self.sfp.send_frame(data)
                        print(f"\n[Sent SFP frame: {bytes_sent} bytes]")
                    except Exception as e:
                        print(f"\n[Error sending SFP frame: {e}]")
                elif user_input == "/quit":
                    self.running = False
                    break
                else:
                    # Send as raw data
                    data = (user_input + '\n').encode('utf-8')
                    self.serial_obj.write(data)
        
        except Exception as e:
            print(f"\nInput thread error: {e}")
            self.running = False
    
    def handle_sfp_frame(self, data):
        print(f"\n{SEP}")
        print(f"SFP FRAME RECEIVED [{len(data)} bytes]:")
        
        try:
            decoded = data.decode('utf-8')
            print(f"Data (string): {decoded}")
        except UnicodeDecodeError:
            hex_data = ' '.join([f"{b:02X}" for b in data])
            print(f"Data (hex): {hex_data}")
        
        print(f"{SEP}")


def main():
    parser = argparse.ArgumentParser(description='Serial Link with SFP Protocol Support')
    parser.add_argument('-p', '--port', type=str, required=True, help='Serial COM Port')
    parser.add_argument('-b', '--baudrate', type=int, default=115200, help='Baudrate')
    parser.add_argument('--no-sfp', action='store_true', help='Disable SFP protocol processing')
    parser.add_argument('-i', '--input', action='store_true', help='Enable input mode')
    args = parser.parse_args()

    SerialLinkSFP(args.port, args.baudrate, not args.no_sfp, args.input)


if __name__ == "__main__":
    main()



