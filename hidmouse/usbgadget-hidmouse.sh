#!/bin/bash

modprobe libcomposite

pushd /sys/kernel/config/usb_gadget

mkdir -p thirdwayv_hidmouse
cd thirdwayv_hidmouse

echo 0x0200 > bcdUSB     # Complies with USB specification 2.0.0
echo 0xFFFE > idVendor   # User specified--your company USB number here
echo 0xFFFE > idProduct  # User specified--your product number here
echo 0x0100 > bcdDevice  # v1.0.0


# String descriptors (configure these before you need them)
mkdir -p strings/0x0409  # 0x0409 is the code for English
echo "AAAAAAAA" > strings/0x0409/serialnumber
echo "Thirdwayv" > strings/0x0409/manufacturer
echo "Thirdwayv Fake Mouse Interface" > strings/0x0409/product

# Configuration descriptor 1
mkdir -p configs/c.1/strings/0x0409
echo "Conf Desc 1" > configs/c.1/strings/0x0409/configuration  # This sets iConfiguration in Configuration Descriptor
echo 500 > configs/c.1/MaxPower  # Request 500 mA

# Careful ... stuff after this line actually calls into the kernel module

mkdir -p functions/hid.usb0		   # Calls *_alloc_inst in the kernel module--we can now set functions specific to the class
echo 2 > functions/hid.usb0/protocol	   # Mouse--specified by USB
echo 1 > functions/hid.usb0/subclass	   # Boot Interface--specified by USB
echo 8 > functions/hid.usb0/report_length  # wMaxPacketSize--set to 8 to enable usage by old low-speed devices

echo -ne \\x05\\x01\\x09\\x02\\xA1\\x01\\x09\\x01\\xA1\\x00\\x05\\x09\\x19\\x01\\x29\\x03\\x15\\x00\\x25\\x01\\x95\\x03\\x75\\x01\\x81\\x02\\x95\\x01\\x75\\x05\\x81\\x01\\x05\\x01\\x09\\x30\\x09\\x31\\x15\\x81\\x25\\x7F\\x75\\x08\\x95\\x02\\x81\\x06\\xC0\\xC0  > functions/hid.usb0/report_desc

# Unpacking the report descriptor using the wonderful "USB Descriptor and Request Parser" at https://eleccelerator.com/usbdescreqparser/
# Descriptor shamelessly copied from: "Appendix E: Example USB Descriptors for HID Class Devices"

# 0x05, 0x01,        // Usage Page (Generic Desktop Ctrls)
# 0x09, 0x02,        // Usage (Mouse)
# 0xA1, 0x01,        // Collection (Application)
# 0x09, 0x01,        //   Usage (Pointer)
# 0xA1, 0x00,        //   Collection (Physical)
# 0x05, 0x09,        //     Usage Page (Button)
# 0x19, 0x01,        //     Usage Minimum (0x01)
# 0x29, 0x03,        //     Usage Maximum (0x03)
# 0x15, 0x00,        //     Logical Minimum (0)
# 0x25, 0x01,        //     Logical Maximum (1)
# 0x95, 0x03,        //     Report Count (3)
# 0x75, 0x01,        //     Report Size (1)
# 0x81, 0x02,        //     Input (Data,Var,Abs,No Wrap,Linear,Preferred State,No Null Position)
# 0x95, 0x01,        //     Report Count (1)
# 0x75, 0x05,        //     Report Size (5)
# 0x81, 0x01,        //     Input (Const,Array,Abs,No Wrap,Linear,Preferred State,No Null Position)
# 0x05, 0x01,        //     Usage Page (Generic Desktop Ctrls)
# 0x09, 0x30,        //     Usage (X)
# 0x09, 0x31,        //     Usage (Y)
# 0x15, 0x81,        //     Logical Minimum (-127)
# 0x25, 0x7F,        //     Logical Maximum (127)
# 0x75, 0x08,        //     Report Size (8)
# 0x95, 0x02,        //     Report Count (2)
# 0x81, 0x06,        //     Input (Data,Var,Rel,No Wrap,Linear,Preferred State,No Null Position)
# 0xC0,              //   End Collection
# 0xC0,              // End Collection

# // 50 bytes

ln -s functions/hid.usb0 configs/c.1/  # calls _alloc in the kernel module

ls /sys/class/udc > UDC  # calls *_bind in the kernel module

# /dev/hidg0 should now exist

# Connecting the USB cable actually calls *_set_alt in the kernel module

popd
