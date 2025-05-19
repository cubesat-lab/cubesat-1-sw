import serial


NL = "\n"
TAB = " " * 4
SEP = "-" * 80


def print_event(obj, event, msg):
    line = f"[{obj}] ({event}): {msg}"
    print(line)


class SerialLink:
    def __init__(self, port, baudrate) -> None:
        self.serial = serial.Serial()
        self.serial.port = port
        self.serial.baudrate = baudrate
        self.serial.timeout = 0.1
        self.serial.open()
        print_event(
            self.__class__.__name__,
            "started",
            f"port = {self.serial.port}, baudrate = {self.serial.baudrate}")

    def read(self):
        if self.serial.is_open:
            byte = self.serial.read()
            if byte:
                # print("R: [{}]".format(hex(ord(byte))))
                return ord(byte)
        return None

    def write(self, byte):
        if self.serial.is_open:
            self.serial.write(byte)
            # print("W: [{}]".format(hex(byte)))

    def close(self):
        self.serial.close()


def main():
    SerialLink("/dev/ttyACM1", 115200)
    SerialLink("/dev/ttyACM0", 115200)


if __name__ == "__main__":
    main()
