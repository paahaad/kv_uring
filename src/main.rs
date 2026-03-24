use libc::*;
use std::{mem, net::Ipv4Addr};

fn main() {
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
            SO_REUSEADDR, 
            &optval as *const _ as *const _ , 
            mem::size_of_val(&optval) as u32
        ) < 0 {
            panic!()
        }

        // Bind to 0.0.0.0:6379:  https://man7.org/linux/man-pages/man3/sockaddr.3type.html
        let addr = sockaddr_in {
            sin_len: 32,
            sin_family: AF_INET as u8,
            sin_port: htons(6379),
            sin_addr: in_addr { s_addr: u32::from_ne_bytes(Ipv4Addr::new(0, 0, 0, 0).octets()).to_be() },
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
        println!("Listening on 0.0.0.0:6370 (fd = {})", fd);

    }
}
