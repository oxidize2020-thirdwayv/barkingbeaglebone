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

use log::{info, warn};

use std::time::Instant;

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

    // Need timing information
    let time_origin = Instant::now();


    const MAX_EPOLL_EVENTS: i32 = 1; // Don't be clever
    const EPOLL_TIMEOUT_MILLIS: i32 = 1000;

    let mut ff_can_write = false;
    let mut ff_can_read = false;
    loop {
        let mut epoll_return_event_struct = libc::epoll_event { events: 0, u64: 0 }; // Empty

        let num_epoll_events_available = syscall!(epoll_wait(
            fd_epoll,
            &mut epoll_return_event_struct,
            MAX_EPOLL_EVENTS,
            EPOLL_TIMEOUT_MILLIS,
        ))?;

        if num_epoll_events_available == 0 {  // Timeout
            if ff_can_write {
                let current_time_millis = Instant::now().duration_since(time_origin).as_millis();

                let bb_report =
                    if (current_time_millis / 1000) % 2 == 0 {
                        b"\x00\xF0\xF0"
                    } else {
                        b"\x00\x0F\x10"
                    };

                match file_usbgadget.write(bb_report) {
                    Ok(nn) => {  // It wrote--nothing to do really ...
                        assert!(nn == 3);
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
                        return Err(ee);  // Unknown error--kick it upstairs
                    }
                }
            } else {
                // All we can do is wait ...
            }
        } else {
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
                ff_can_read = true;
                warn!("Oops.  Something attempted to send to us");
            }
        }
    }
}
