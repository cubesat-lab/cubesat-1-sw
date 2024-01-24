import argparse
import help_serial


parser = argparse.ArgumentParser(description='A tool to establish serial link with NUCLEO-F767ZI board')
parser.add_argument('-p', '--port', type=str, required=True, help='Serial COM Port')
parser.add_argument('-b', '--baudrate', type=int, default=115200, help='Baudrate')
args = parser.parse_args()

my_serial = help_serial.SerialLink(args.port, args.baudrate)
my_frame = 'aaaa0002aabbc860'
byte_data = bytes.fromhex(my_frame)

my_serial.send_frame(byte_data)
my_serial.listen_frame()
my_serial.close_serial()
