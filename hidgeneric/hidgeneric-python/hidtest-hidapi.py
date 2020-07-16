#!/usr/bin/env python3

import hid  # Grabs https://github.com/apmorton/pyhidapi
import time

# This example file is to demonstrate driver is working on systems which will not
# relinquish the USB device.

MAX_USB_REPORT_SIZE = 64  # Needs to match the report descriptor we set outside

def main():
    vid = 0xFFFE	# Change it for your device
    pid = 0xFFFE	# Change it for your device

    with hid.Device(vid, pid) as hd:
        print(f'Device manufacturer: {hd.manufacturer}')
        print(f'Product: {hd.product}')
        print(f'Serial Number: {hd.serial}')

        ui = 0
        while ui < 10:
            ba_hid_out = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ"
            
            hd.write(ba_hid_out)

            ss = [chr(cc) for cc in ba_hid_out]
            print("Sent    :", "".join(ss))

            ba_hid_in = hd.read(MAX_USB_REPORT_SIZE)

            ss = [chr(cc) for cc in ba_hid_in if ((cc > 32) and (cc < 127))]
            print("Received:", "".join(ss))

            time.sleep(1.000)

            ui += 1
        
if __name__ == "__main__":
    main()
