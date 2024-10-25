#!/usr/bin/env python

import socket
import struct
import sys

BYTE_ORDER='little'
RECV_BUFFSIZE=2048

#S-> server sends
#<-C client sends
#
#Client connect & Game setup messages
#S-> game message
#S-> version message
#<-C character message (create character)
#<-C start
#
#
#Status messages:
#S-> Message (can be sent at any time)
#S-> Accept (or error)
#S-> Error (or accept)
#
#Gameplay messages
#S-> room, Character (for player and all other characters in the room)
#<-C Change Room
#S-> Connection
#<-C fight
#<-C PVP Fight
#<-C Loot
#<-C Leave
#


def send_character(character_name, attack, defense, regen, description, skt):
    send_bytes = b'\x0a'    # messages type 10

    # truncate character name to 32 bytes, then pad with '\0' if necessary
    character_name = character_name[:32] if len(character_name) > 32 else character_name
    send_bytes += bytes(character_name.ljust(32, '\0'), encoding='utf-8')

    send_bytes += b'\x88'   #character flags; 88 = 'alive' and 'ready' set
    send_bytes += bytes(struct.pack('<H', attack))   # pack attack into 'unsigned short' (two bytes)
    send_bytes += bytes(struct.pack('<H', defense))  # pack 'defense' into 'unsigned short'
    send_bytes += bytes(struct.pack('<H', regen))    # pack 'regen' into 'unsigned short'
    send_bytes += bytes(struct.pack('<h', 0))        # placeholder 'health' value required by protocol
    send_bytes += bytes(struct.pack('<H', 0))        # placholder 'gold' value required by protocol
    send_bytes += bytes(struct.pack('<H', 0))        # placeholder 'room' value required by protocol

    # set character description (variable length)
    desc_len = len(description)
    send_bytes += bytes(struct.pack('<h', desc_len))
    send_bytes += bytes(description, encoding='utf-8')

    print(f"Byte 0:     {send_bytes[0]}")
    print(f"Byte 1-32:  {send_bytes[1:32].decode('utf-8')}")
    print(f"Byte 33:    {bin(int(send_bytes[33]))}")
    print(f"Byte 34-35: {int.from_bytes(send_bytes[34:35])}")
    print(f"Byte 36-37: {int.from_bytes(send_bytes[36:37])}")
    print(f"Byte 38-39: {int.from_bytes(send_bytes[38:39])}")
    print(f"Byte 40-41: {int.from_bytes(send_bytes[40:41])}")
    print(f"Byte 42-43: {int.from_bytes(send_bytes[42:43])}")
    print(f"Byte 44-45: {int.from_bytes(send_bytes[44:45])}")
    print(f"Byte 46-47: {int.from_bytes(send_bytes[46:47])}")
    print(f"bytes 48-desc_len: {send_bytes[48:(48+desc_len)].decode('utf-8')}")
    skt.send(send_bytes)

def send_start(skt):
    send_bytes = b"\x06"
    print("Sending Start message")
    print(f"byte 0: {send_bytes[0]}")
    skt.send(send_bytes)


def handle_game_msg(msg):
    print("Handling game message")
    print(f"initial points: {int.from_bytes(msg[1:3], BYTE_ORDER)}")
    print(f"stat limit: {int.from_bytes(msg[3:5], BYTE_ORDER)}")
    desc_len = int.from_bytes(msg[5:7], BYTE_ORDER)
    print(f"desc length: {desc_len}")
    print(f"desc: {msg[7:7+desc_len].decode('utf-8')}")

def handle_version_msg(msg):
    print("Handling version message")
    print(f"message type: {msg[0]}")
    print(f"major version: {msg[1]}")
    print(f"minor version: {msg[2]}")
    print(f"extension len: {int.from_bytes(msg[3:5], BYTE_ORDER)}")

def handle_accept(msg):
    print("Handling accept message")
    print(f"message type: {msg[0]}")
    print(f"type of accepted action: {msg[1]}")

def handle_error(msg):
    codes_dict = {
            0 : "Other",
            1 : "Bad Room",
            2 : "Player exists",
            3 : "Bad monster",
            4 : "Stat error",
            5 : "Not Ready",
            6 : "No target",
            7 : "No Fight",
            8 : "No PVP on Server",
    }
    print("handling error message")
    print(f"message type: {msg[0]}")
    error_code = msg[1]
    print(f"error code: {error_code}")
    message_len = int.from_bytes(msg[2:4], BYTE_ORDER)
    print(f"error message length: {message_len}")
    print(f"error message {msg[4:4+message_len].decode('utf-8')}")
    print(f"error code {error_code} means {codes_dict[error_code]}")

def main():
    skt = socket.socket();
    skt.connect( (str(sys.argv[1]), int(sys.argv[2])) )

if __name__=="__main__":
    skt = socket.socket();
    skt.connect( (str(sys.argv[1]), int(sys.argv[2])) )
    messages = []

    # for this test, expect a 'game' and a 'version' message
    messages.append(skt.recv(RECV_BUFFSIZE))
    messages.append(skt.recv(RECV_BUFFSIZE))
    for recv_msg in messages:
        if(recv_msg[0] == 11):
            handle_game_msg(recv_msg)
        if(recv_msg[0] == 14):
            handle_version_msg(recv_msg)
    messages = []

    # send character, expect 'accept' or 'error' in response
    send_character(character_name="ohai",
                   attack=50,
                   defense=50,
                   regen=50,
                   description="this is my character I am a character",
                   skt=skt)
    messages.append(skt.recv(RECV_BUFFSIZE))
    for recv_msg in messages:
        if(recv_msg[0] == 8):
            handle_accept(recv_msg)
        if(recv_msg[0] == 7):
            handle_error(recv_msg)
    messages = []

    # sdend start; expect 'accept', 'error', 'room' and 1 or more 'character' messages
    print("sending start")
    send_start(skt)
    recv_msg = "notnone"
    print(f"messages length: {len(messages)}")
    while(recv_msg != ""):
        recv_msg = skt.recv(RECV_BUFFSIZE)
        if recv_msg != "":
            messages.append(recv_msg)
    print(messages)


    skt.close()
