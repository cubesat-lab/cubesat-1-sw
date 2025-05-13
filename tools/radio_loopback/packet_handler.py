from enum import IntEnum
from serial_link import SerialLink
import time

# Protocol Host <-> RF Device
    # The story:
    # Hey, I'm going to send you a 64 bytes packet
    # Okay
    # ...sending packet ...packet sent
    # I have received the packet, and will shortly send it via RF
    # I have sent the packet via RF, I am ready to receive a new packet
    #
    # Mnemonics:
    # Host              Device          State
    # ------------------------------------------------
    #                                   idle
    # PKT_REQ ->                        pkt_req
    #                <- PKT_REQ_ACK     pkt_req_ack
    # (pkt) ->                          pkt_tx
    #                <- PKT_ACK         pkt_ack
    #                <- PKT_SENT_RF     pkt_sent_rf
    # ------------------------------------------------

# Mnemonics:
# PKT_REQ       0b1100_0011     0xC3
# PKT_REQ_ACK   0b1100_0110     0xC6
# PKT_ACK       0b1100_1100     0xCC
# PKT_SENT_RF   0b1101_1000     0xD8

# class State(IntEnum):
#     idle = 1
#     pkt_req = 2
#     pkt_req_ack = 3
#     pkt_tx = 4
#     pkt_ack = 5
#     pkt_sent_rf = 6


PACKET_LEN = 64


class Protocol(IntEnum):
    PKT_REQ     = 0b1100_0011   # 0xC3
    PKT_REQ_ACK = 0b1100_0110   # 0xC6
    PKT_ACK     = 0b1100_1100   # 0xCC
    PKT_SENT_RF = 0b1101_1000   # 0xD8
    PKT_RCV_RF  = 0b1111_0000   # 0xF0


class Result(IntEnum):
    Success = 0
    Timeout = 1


class PacketHandler:
    def __init__(self, serial: SerialLink):
        self.serial = serial

    def send(self, packet: list, timeout=1) -> Result:

        self.serial.write(Protocol.PKT_REQ)

        if not self.__read_expected_with_timeout(expect=Protocol.PKT_REQ_ACK, timeout=0.2):
            return Result.Timeout

        self.__send_packet(packet)

        if not self.__read_expected_with_timeout(expect=Protocol.PKT_ACK, timeout=0.2):
            return Result.Timeout

        if not self.__read_expected_with_timeout(expect=Protocol.PKT_SENT_RF, timeout=0.2):
            return Result.Timeout

        return Result.Success

    def __read_expected_with_timeout(self, expect, timeout):
        start_time = time.time()

        while True:
            response = self.serial.read()

            # Check if the expected response was read from serial
            if response and (response == expect):
                return True

            # Check if the timeout has been reached
            if (time.time() - start_time) > timeout:
                return False

    def __send_packet(self, packet):
        for value in packet:
            self.serial.write(value)

    def receive(self) -> list:
        packet = []
        index = 0

        while True:
            if self.serial.read() != Protocol.PKT_RCV_RF:
                continue

            while True:
                read_byte = self.serial.read()
                if not read_byte:
                    continue

                packet[index] = read_byte
                index += 1

                if index == PACKET_LEN:
                    return packet
