
# Built-in libs
import os
import argparse
import logging
import time
from threading import Thread, Event

# App's includes
from serial_link import SerialLink


PACKET_LEN = 64  # bytes


class SerialCom:
    def __init__(self, args, logger):
        self.args = args
        self.serial_link = SerialLink(args.port, args.baudrate)
        logger.info(f"Serial link created")

    def run(self):
        self.start_threads()

        # Main thread
        while True:
            # Wait forever
            Event().wait()

    def start_threads(self):
        self.threads = list()
        thread_list = [
            self.thread_serial,
            self.thread_com_process,
        ]

        for thread_func in thread_list:
            thread_obj = Thread(target=thread_func, args=(1,), daemon=True)
            thread_obj.start()
            self.threads.append(thread_obj)

    def thread_serial(self, *args, **kwargs):
        while True:
            if self.args.test == "loopback_usb":
                self.loopback_test()
            else:
                print("Unknown test type")
                break

    def loopback_test(self):
        min_sleep = 0.0001
        max_sleep = 1.0
        sleep_time = 1
        error_count = 0
        ok_count = 0

        start_time = time.time()

        time.sleep(sleep_time)

        packet_send = self.build_packet()
        # print("Sending packet:  {}".format(" ".join(["{:02X}".format(x) for x in packet_send])))
        self.send_packet(packet_send)

        packet_received = self.receive_packet()
        # print("len: {}".format(len(packet_received)))
        if packet_received:
            # print("Received packet: {}".format(" ".join(["{:02X}".format(x) for x in packet_received])))
            packet_test = self.build_packet()
            packet_test = [x + 0 for x in packet_test]
            if packet_received != packet_test:
                error_count += 1
                ok_count = 0
                print("Received packet: {}".format(" ".join(["{:02X}".format(x) for x in packet_received])))
                print("Expected packet: {}".format(" ".join(["{:02X}".format(x) for x in packet_test])))
            else:
                ok_count += 1
                error_count = 0
                # print("Ok")
        else:
            error_count += 1
            ok_count = 0
            print("No packet received")

        end_time = time.time()
        elapsed_time = end_time - start_time
        data_rate = (PACKET_LEN * 2) / elapsed_time
        print("Data rate: {:.1f} b/s | time {:.1f} ms | sleep {:.1f}".format(data_rate, elapsed_time * 1000, sleep_time * 1000))

        # Controller logic
        if error_count > 0:
            # If error, slow down
            sleep_time = min(sleep_time * 1.5, max_sleep)
        elif ok_count > 2:
            # If several successes, speed up
            sleep_time = max(sleep_time * 0.7, min_sleep)
            ok_count = 0  # Reset after adjustment

    def thread_com_process(self, *args, **kwargs):

        while True:
            time.sleep(1)

    @staticmethod
    def build_packet():
        packet = [0] * PACKET_LEN

        for i in range(PACKET_LEN):
            packet[i] = i & 0xFF

        return packet

    def send_packet(self, packet):
        self.serial_link.write(packet)

    def receive_packet(self) -> list:
        packet = []

        while True:
            read_byte = self.serial_link.read()

            if read_byte == None:
                return packet
            else:
                packet.append(read_byte)


def setup_logger(name):
    log_filename = __file__.replace(__name__ + ".py", name + ".log")

    # Create a logger
    logger = logging.getLogger(name)
    logger.setLevel(logging.INFO)

    # Create a file handler that writes to the log file in 'write' mode
    file_handler = logging.FileHandler(log_filename, mode='w')
    file_handler.setLevel(logging.INFO)  # Set the logging level for the handler

    # Create a formatter and set it for the file handler
    formatter = logging.Formatter('%(asctime)s - %(name)s - %(levelname)s - %(message)s')
    file_handler.setFormatter(formatter)

    # Add the file handler to the logger
    logger.addHandler(file_handler)

    return logger


def app_args():
    parser = argparse.ArgumentParser(description='A tool to perform Serial Communication')
    parser.add_argument('-p', '--port', type=str, required=True, help='Serial port')
    parser.add_argument('-b', '--baudrate', type=int, default=115200, help='Baudrate')
    parser.add_argument('-t', '--test', type=str, choices=['loopback_usb', 'loopback_serial'], default='loopback_usb', help='Test kind')
    return parser.parse_args()


def main():
    app_name = os.path.basename(os.path.dirname(os.path.abspath(__file__)))
    logger = setup_logger(app_name)
    args = app_args()

    logger.info(f"Starting '{app_name}'")

    app = SerialCom(args, logger)

    try:
        logger.info(f"Running '{app_name}'")
        app.run()
    except KeyboardInterrupt:
        logger.info("KeyboardInterrupt received. Exiting...")
        app.serial_link.close()
        logger.info("Serial link closed")
    except Exception as e:
        logger.error(f"An error occurred: {e}")
    finally:
        logger.info("Exiting application")


if __name__ == "__main__":
    main()
