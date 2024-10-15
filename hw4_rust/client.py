#!/usr/bin/env python

import socket
import struct
import sys

def main():
    skt = socket.socket();
    skt.connect( (str(sys.argv[1]), int(sys.argv[2])) )

    # see struct.pack format types:
    # https://docs.python.org/3/library/struct.html#format-characters 
    # using '<d' to specify 'little endian double' (defined as 8 bytes and matching f64 on the Rust side)
    request_type = input("enter request type by number: (1) set value, (2) convert sheep to dollars, (3) convert dollars to sheep: ")
    print(f"Request type is: {request_type}")
    if request_type == "1":
        new_sheep_val = float(input("Set the dollar value of a sheep: "))
        send_bytes = b'\x01'
        send_bytes += bytes(struct.pack('<d', new_sheep_val))
        skt.send(send_bytes)
        blah = skt.recv(1024)
        if blah == b'\x04':
            print("It was OK")
        else:
            print("It was not OK")
    elif request_type == "2":
        number_of_sheep = float(input("converting sheep to dollars; how many sheep do you have? "))
        send_bytes = b'\x02'
        send_bytes += bytes(struct.pack('<d', number_of_sheep));
        skt.send(send_bytes)
        in_data = skt.recv(1024)
        _ = in_data[0] #message type, ignored
        [num_dollars] = struct.unpack('d', in_data[1:])
        print(f"You now have ${num_dollars} dollhairs")
    elif request_type == "3":
        number_of_dollars = float(input("converting dollhairs to sheepz; how many dollhairs do you have? "))
        send_bytes = b'\x03'
        send_bytes += bytes(struct.pack('<d', number_of_dollars));
        skt.send(send_bytes);
        in_data = skt.recv(1024)
        _ = in_data[0] #message type, ignored
        [num_sheepz] = struct.unpack('d', in_data[1:])
        print(f"You now have approximately {num_sheepz} sheepz")
    else:
        print("Sorry, I didn't understand that input. Try running me again.")

    skt.close()

if __name__=="__main__":
    main()
