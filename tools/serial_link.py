import serial
import argparse

"""
How to use serial module besides python scrypt:
    python -m serial.tools.list_ports -v
    python -m serial.tools.miniterm <com_port> <baudrate>
"""

NL = "\n"
TAB = " " * 4
SEP = "-" * 80


class SerialLink:
    def __init__(self, port, baudrate) -> None:
        with serial.Serial() as serial_obj:
            serial_obj.port = port
            serial_obj.baudrate = baudrate
            serial_obj.timeout = 0.1
            serial_obj.open()

            print("Started serial link:")
            print(f"{TAB}Port:     {serial_obj.port}")
            print(f"{TAB}Baudrate: {serial_obj.baudrate}")
            print(f"{SEP}")

            try:
                while serial_obj.is_open:
                    line = serial_obj.readline().decode("utf-8").replace("\n", "")
                    if line:
                        print(f"{line}")
            except KeyboardInterrupt:
                print(f"{SEP}{NL}User requested exit")
            except:
                print(f"{SEP}{NL}Serial port closed")


def main():
    parser = argparse.ArgumentParser(description='A tool to establish serial link with NUCLEO-F767ZI board')
    parser.add_argument('-p', '--port', type=str, required=True, help='Serial COM Port')
    parser.add_argument('-b', '--baudrate', type=int, default=115200, help='Baudrate')
    args = parser.parse_args()

    SerialLink(args.port, args.baudrate)


if __name__ == "__main__":
    main()
