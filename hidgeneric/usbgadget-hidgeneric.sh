#!/bin/bash

modprobe libcomposite

pushd /sys/kernel/config/usb_gadget

mkdir -p thirdwayv_hidgeneric
cd thirdwayv_hidgeneric

echo 0x0200 > bcdUSB     # Complies with USB specification 2.0.0
echo 0xFFFE > idVendor   # User specified--your company USB number here
echo 0xFFFE > idProduct  # User specified--your product number here
echo 0x0100 > bcdDevice  # v1.0.0


# String descriptors (configure these before you need them)
mkdir -p strings/0x0409  # 0x0409 is the code for English
echo "AAAAAAAA" > strings/0x0409/serialnumber
echo "Thirdwayv" > strings/0x0409/manufacturer
echo "Thirdwayv HID Generic Interface" > strings/0x0409/product

# Configuration descriptor 1
mkdir -p configs/c.1/strings/0x0409
echo "Conf Desc 1" > configs/c.1/strings/0x0409/configuration  # This sets iConfiguration in Configuration Descriptor
echo 500 > configs/c.1/MaxPower  # Request 500 mA

# Careful ... stuff after this line actually calls into the kernel module

mkdir -p functions/hid.usb0  # Calls *_alloc_inst in the kernel module--we can now set functions specific to the class
echo 0 > functions/hid.usb0/protocol  # No protocol
echo 0 > functions/hid.usb0/subclass  # No boot interface
echo 64 > functions/hid.usb0/report_length # wMaxPacketSize--set to 64 to enable usage by full-speed devices

echo -ne \\x06\\x00\\xFF\\x09\\x01\\xA1\\x01\\x19\\x01\\x29\\x40\\x15\\x00\\x26\\xFF\\x00\\x75\\x08\\x95\\x40\\x81\\x00\\x19\\x01\\x29\\x40\\x91\\x00\\xC0  > functions/hid.usb0/report_desc

# Unpacking the report descriptor using the wonderful "USB Descriptor and Request Parser" at https://eleccelerator.com/usbdescreqparser/
# Descriptor shamelessly copied from: "Appendix E: Example USB Descriptors for HID Class Devices"

# 0x06, 0x00, 0xFF,  // Usage Page (Vendor Defined 0xFF00)
# 0x09, 0x01,        // Usage (0x01)
# 0xA1, 0x01,        // Collection (Application)
# 0x19, 0x01,        //   Usage Minimum (0x01)
# 0x29, 0x40,        //   Usage Maximum (0x40)
# 0x15, 0x00,        //   Logical Minimum (0)
# 0x26, 0xFF, 0x00,  //   Logical Maximum (255)
# 0x75, 0x08,        //   Report Size (8)
# 0x95, 0x40,        //   Report Count (64)
# 0x81, 0x00,        //   Input (Data,Array,Abs,No Wrap,Linear,Preferred State,No Null Position)
# 0x19, 0x01,        //   Usage Minimum (0x01)
# 0x29, 0x40,        //   Usage Maximum (0x40)
# 0x91, 0x00,        //   Output (Data,Array,Abs,No Wrap,Linear,Preferred State,No Null Position,Non-volatile)
# 0xC0,              // End Collection

# // 29 bytes


ln -s functions/hid.usb0 configs/c.1/  # calls _alloc in the kernel module

ls /sys/class/udc > UDC  # calls *_bind in the kernel module

# /dev/hidg0 should now exist

# Connecting the USB cable actually calls *_set_alt in the kernel module

popd


