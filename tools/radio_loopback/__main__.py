
# Built-in libs
import os
import argparse
import logging
import time
from threading import Thread, Event

# App's includes
from serial_link import SerialLink
from packet_handler import PacketHandler, PACKET_LEN


class RadioLoopback:
    def __init__(self, args, logger):

        self.packet_handler_sender = PacketHandler(SerialLink(args.port_sender, args.baudrate))
        # self.packet_handler_receiver = PacketHandler(SerialLink(args.port_receiver, args.baudrate))

        logger.info(f"Packet handlers created")

        self.start_threads()

        # Main thread
        while True:
            # Wait forever
            Event().wait()


    def start_threads(self):
        self.threads = list()
        thread_list = [
            self.thread_packet_send,
            self.thread_packet_receive,
            self.thread_loopback_process,
        ]

        for thread_func in thread_list:
            thread_obj = Thread(target=thread_func, args=(1,), daemon=True)
            thread_obj.start()
            self.threads.append(thread_obj)


    def thread_packet_send(self, *args, **kwargs):
        counter = 1
        # Wait for a signal to start work for N seconds
        # Set counter=1
        while True:
            time.sleep(1)

            packet = self.build_packet(counter)
            # print("packet: {}".format(packet))
            # print("counter: {}".format(self.get_counter_from_packet(packet)))
            result = self.packet_handler_sender.send(packet, timeout=0.2)
            # print(result)

            counter += 1

            # Create packet(counter=1)
            # Send the packet to lowelayer (save timestamp)
            # Wait for timeout or success (timeout=200ms)
            # Report timestamp or timeout


    def thread_packet_receive(self, *args, **kwargs):
        while True:
            # print("2")
            time.sleep(1)

            # packet = self.packet_handler_receiver.receive()

            # Wait for a packet to be received
            # When/if packet received, extract counter, save timestamp, report these data

    def thread_loopback_process(self, *args, **kwargs):
        # Ask thread_packet_send to start sending packets with incrementing counter for N seconds

        while True:
            # print("3")
            time.sleep(1)

            # self.counter
            # Wait from thread_packet_send a timestamp for a msg


        # Initialize serial_links and serial_engines

        # Send data to selected device
            # Generate specific data with frames of 64 bytes
            # Perform acknoledges checks before sending other data

        # Wait for the data from other device
            # Read and check received data
            # Compare with expected payload
            # Check losses in transmission

        # Check the round-trip time of a packet (ping)
        # Check how much packets (data) were transmitted in a window of time

    @staticmethod
    def build_packet(counter):
        assert (counter <= 0xFFFFFFFF) and (counter >= 0), "Counter exceeds uint32 type value"

        packet = [0] * PACKET_LEN

        for i in range(8):
            # Extract the i-th nibble
            nibble = (counter >> (i * 4)) & 0xF

            # Place the nibble in the lower 4 bits of the i-th byte (in big-endian order)
            packet[(8 - 1) - i] = nibble

        return packet

    @staticmethod
    def get_counter_from_packet(packet):
        assert isinstance(packet, list) and (len(packet) == 64) and isinstance(packet[0], int), "Packet must be a list of 64 integers"

        counter = 0

        for i in range(8):
            # Extract the nibble from the packet. Reverse the logic of how the packet was built
            nibble = packet[(8 - 1) - i]

            # Place the nibble in the correct position in the counter
            counter |= (nibble << (i * 4))

        return counter


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
    parser = argparse.ArgumentParser(description='A tool to perform Radio Loopback using 2 RF devices connected to serial ports')
    parser.add_argument('-ps', '--port-sender', type=str, required=True, help='Serial port for device which sends packet on RF')
    parser.add_argument('-pr', '--port-receiver', type=str, required=True, help='Serial port for device which receives packet on RF')
    parser.add_argument('-b', '--baudrate', type=int, default=115200, help='Baudrate')
    return parser.parse_args()


def main():
    app_name = os.path.basename(os.path.dirname(os.path.abspath(__file__)))
    logger = setup_logger(app_name)
    args = app_args()

    logger.info(f"Starting '{app_name}'")

    RadioLoopback(args, logger)


if __name__ == "__main__":
    main()
