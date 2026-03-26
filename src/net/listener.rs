use libc::*;
use std::mem;

pub fn create_listener() -> i32 {
    unsafe {
        // Create socket fd: https://man7.org/linux/man-pages/man2/socket.2.html
        let fd = libc::socket(AF_INET, SOCK_STREAM | SOCK_NONBLOCK, 0);
        if fd < 0 {
            panic!("Socket Connection Failed")
        }

        // Enable SO_REUSEADDR: https://man7.org/linux/man-pages/man3/setsockopt.3p.html
        let optval: i32 = 1;
        if setsockopt(
            fd,
            SOL_SOCKET,
            SO_REUSEADDR, // allow bind even if port is in TIME_WAIT state
            &optval as *const _ as *const _,
            mem::size_of_val(&optval) as u32
        ) < 0 {
            panic!()
        }

        // Bind to 0.0.0.0:6379:  https://man7.org/linux/man-pages/man3/sockaddr.3type.html
        let addr = sockaddr_in {
            sin_family: AF_INET as u16,
            sin_port: htons(6379),
            sin_addr: in_addr { s_addr: INADDR_ANY }, // INADDR_ANY is 0u32, already network byte order
            sin_zero: [0; 8],
        };

        if bind(
            fd,
            &addr as *const _ as *const sockaddr,
            mem::size_of_val(&addr) as u32
        ) < 0 {
            panic!("Build Failed");
        }

        // Listen
        if listen(fd, 128) < 0 {
            panic!("Listen Failed");
        }

        fd
    }
}
