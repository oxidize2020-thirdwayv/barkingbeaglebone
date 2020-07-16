// Shamelessly stolen from tokio ...
macro_rules! syscall {
    ($fn: ident ( $($arg: expr),* $(,)* ) ) => {{
        let res = unsafe { libc::$fn($($arg, )*) };
        if res == -1 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(res)
        }
    }};
}

use bytes::{Bytes, BytesMut, Buf, BufMut};

use log::{debug, error, info, warn};

use std::fs::OpenOptions;

use std::io::{self, Read, Write};

use std::os::unix::io::AsRawFd;


fn main() -> io::Result<()> {
    env_logger::init();

    info!("Logger started");

    let mut file_usbgadget = OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/hidg0")?;

    // Switch the character device file descriptors to non-blocking
    let old_flags = syscall!(fcntl(file_usbgadget.as_raw_fd(), libc::F_GETFL, 0))?;
    syscall!(fcntl(file_usbgadget.as_raw_fd(), libc::F_SETFL, old_flags | libc::O_NONBLOCK))?;

    // Set up epoll infrastructure
    let fd_epoll = syscall!(epoll_create1(0))?;

    let mut epoll_events_to_watch = libc::epoll_event {
        events: (libc::EPOLLIN as u32 | libc::EPOLLOUT as u32),
        u64: file_usbgadget.as_raw_fd() as u64, // Careful--this is a union on the Linux side but Rust libc doesn't do that
    };

    syscall!(epoll_ctl(
        fd_epoll,
        libc::EPOLL_CTL_ADD,
        file_usbgadget.as_raw_fd(),
        (&mut epoll_events_to_watch) as *mut libc::epoll_event
    ))?;


    const MAX_EPOLL_EVENTS: i32 = 1; // Don't be clever
    const EPOLL_TIMEOUT_MILLIS: i32 = 10000;  // 10 seconds

    const MAX_USB_REPORT_SIZE: usize = 64;  // Needs to match the report descriptor we set outside

    let mut ff_can_write = false;
    let mut ff_write_buffer_filled = false;

    let mut bb_array_usb_read = [0u8; MAX_USB_REPORT_SIZE];
    let mut bb_usb_report_write = BytesMut::with_capacity(MAX_USB_REPORT_SIZE);

    info!("Entering loop");

    loop {
        let mut ff_can_read = false;

        let mut epoll_return_event_struct = libc::epoll_event { events: 0, u64: 0 }; // Empty

        let num_epoll_events_available = syscall!(epoll_wait(
            fd_epoll,
            &mut epoll_return_event_struct,
            MAX_EPOLL_EVENTS,
            EPOLL_TIMEOUT_MILLIS,
        ))?;

        if num_epoll_events_available != 0 {
            info!("Events available");

            assert!(
                epoll_return_event_struct.u64 == file_usbgadget.as_raw_fd() as u64,
                "Epoll returned strange descriptor: {}",
                epoll_return_event_struct.u64
            );

            if (epoll_return_event_struct.events & (libc::EPOLLOUT as u32)) != 0 {
                info!("Send enabled");

                ff_can_write = true;

                // Shut down EPOLLOUT for now or it will keep spamming us
                epoll_events_to_watch.events = libc::EPOLLIN as u32; // Read event only--remove write
                syscall!(epoll_ctl(
                    fd_epoll,
                    libc::EPOLL_CTL_MOD,
                    file_usbgadget.as_raw_fd(),
                    (&mut epoll_events_to_watch) as *mut libc::epoll_event))?;
            }

            if (epoll_return_event_struct.events & (libc::EPOLLIN as u32)) != 0 {
                // We don't shut the descriptor down as that is handled by read-ing the buffer
                ff_can_read = true;

                info!("Read requested");
            }
        } else {
            info!("Timeout");
            // Timeout.  Do nothing.
        }

        if ff_can_read {
            // FIXME: CAUTION: Rust conflates Ok(0) with WouldBlock and that screws up epoll
            match syscall!(read(file_usbgadget.as_raw_fd(), bb_array_usb_read.as_mut_ptr() as *mut libc::c_void, MAX_USB_REPORT_SIZE)) {
                Ok(nn) => {
                    info!("Read: {}", nn);

                    // FIXME: Need a much better way to do this ...

                    // CAUTION: HID drivers can be pedantic, if you specify 64 bytes as your
                    // CAUTION: report size, they may demand *exactly* that or toss the packet
                    // CAUTION: (OS X is apprently persnickety about this, for example)
                    bb_usb_report_write.resize(MAX_USB_REPORT_SIZE as usize, 0x00);
                    for ui in 0..MAX_USB_REPORT_SIZE {
                        bb_usb_report_write[ui] = 0x00;
                    }
                    for ui in 0..nn as usize {  // FIXME: Better way to do this?
                        bb_usb_report_write[ui] = bb_array_usb_read[ui].wrapping_add(13);
                    }

                    ff_write_buffer_filled = true;
                }
                Err(ee) if ee.kind() == io::ErrorKind::WouldBlock => {
                    // Do nothing as there is nothing to read
                }
                Err(ee) => {
                    error!("Read exception");
                    Err(ee)?;
                }
            }

            info!("Read finished");
        }

        if ff_can_write && ff_write_buffer_filled {
            match file_usbgadget.write(&bb_usb_report_write) {
                Ok(nn) => {  // It wrote--nothing to do really ...
                    assert!(nn == bb_usb_report_write.len());
                    info!("Send USB packet of length: {}", nn);

                    ff_write_buffer_filled = false;
                }
                Err(ee) if ee.kind() == io::ErrorKind::WouldBlock => {
                    // Write failed--most likely unplugged the device
                    ff_can_write = false;
                    epoll_events_to_watch.events =
                        libc::EPOLLIN as u32 | 
                        libc::EPOLLOUT as u32;  // Re-enable write 
                    syscall!(epoll_ctl(
                        fd_epoll,
                        libc::EPOLL_CTL_MOD,
                        file_usbgadget.as_raw_fd(),
                        (&mut epoll_events_to_watch) as *mut libc::epoll_event))?;
                }
                Err(ee) => {
                    error!("Read exception");
                    return Err(ee);  // Unknown error--kick it upstairs
                }
            }
        } 

    }

}
