
import time

import usb.core
import usb.util

MAX_USB_REPORT_SIZE = 64  # Needs to match the report descriptor we set outside                                                                                                                     

def main():
    # find our device                                                                                                                                                                               
    hid_device = usb.core.find(idVendor=0xfffe, idProduct=0xfffe)
    assert hid_device is not None

    if hid_device.is_kernel_driver_active(0):
        try:
            hid_device.detach_kernel_driver(0)
            print("Detached device from kernel driver")
        except usb.core.USBError as ee:
            print("Could not detach kernel driver")
            raise ee
    else:
        print("No kernel driver attached")

    try:
        usb.util.claim_interface(hid_device, 0)
        print("Claimed device")
    except usb.core.USBError as ee:
        print("Could not claim device")
        raise ee

    try:
        hid_device.reset()  # Needs to occur *before* set_configuration for some reason ...                                                                                                         
        hid_device.set_configuration()
        hid_device.reset()
        print("Set configuration")
    except usb.core.USBError as ee:
        print("Could not set configuration")
        raise ee

    # get an endpoint instance                                                                                                                                                                      
    hid_config = hid_device.get_active_configuration()
    hid_interface = hid_config[(0, 0)]

    hid_endpoint_in  = hid_device[0][(0, 0)][0]
    hid_endpoint_out = hid_device[0][(0, 0)][1]
    print("EPS:", hid_endpoint_in)
    print("EPS:", hid_endpoint_out)

    ba_hid_out = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ"

    while 1:
        try:
            nn = hid_endpoint_out.write(ba_hid_out)

            ss = [chr(cc) for cc in ba_hid_out[0:nn]]
            print("Sent    :", "".join(ss))

        except usb.core.USBError as ee:
            print("Write USB Error")

        try:
            ba_hid_in = hid_endpoint_in.read(64)

            ss = [chr(cc) for cc in ba_hid_in]
            print("Received:", "".join(ss))

        except usb.core.USBError as ee:
            print("Read USB Error")

        time.sleep(1.000)


if __name__ == "__main__":
    main()

