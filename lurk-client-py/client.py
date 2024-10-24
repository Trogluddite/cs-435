#!/usr/bin/env python

import socket
import struct
import sys


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

    print(desc_len, description)
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
    #rcv_buf = skt.recv(1024)
    #print(rcv_buf)

def handle_game_msg(msg):
    print("Handling game message")
    print(f"message type: {msg[0]}")
    print(f"initial points: {int.from_bytes(msg[1:2])}")
    print(f"stat limit: {int.from_bytes(msg[3:4])}")
    desc_len = int.from_bytes(msg[5:6])
    print(f"desc length: {desc_len}")
    print(f"desc: {msg[7:7+desc_len].decode('utf-8')}")

def handle_version_msg(msg):
    print("Handling version message")
    print(f"message type: {msg[0]}")
    print(f"major version: {msg[1]}")
    print(f"minor version: {msg[2]}")
    print(f"extension len: {int.from_bytes(msg[3:4])}")

def main():
    skt = socket.socket();
    skt.connect( (str(sys.argv[1]), int(sys.argv[2])) )

if __name__=="__main__":
    skt = socket.socket();
    skt.connect( (str(sys.argv[1]), int(sys.argv[2])) )
    recv_msg = skt.recv(4096)

    while(len(recv_msg) > 0):
        if(recv_msg[0] == 11):
            msg_len = 6
            desc_len = int.from_bytes(recv_msg[5:6])
            print(desc_len)
            msg_len += desc_len
            handle_msg = recv_msg[1:msg_len]
            recv_msg = recv_msg[msg_len+1:]
            handle_game_msg(handle_msg)
        if(recv_msg[0] == 14):
            msg_len = 4
            extension_len = int.from_bytes(recv_msg[3:4])
            msg_len += extension_len
            handle_msg = recv_msg[0:msg_len]
            recv_msg = recv_msg[msg_len+1:]
            handle_version_msg(handle_msg)

    send_character(character_name="ohai",
                   attack=500,
                   defense=500,
                   regen=500,
                   description="this is my character I am a character",
                   skt=skt)
    skt.close()
