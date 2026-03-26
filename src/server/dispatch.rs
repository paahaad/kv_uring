use std::io;

use io_uring::{IoUring, cqueue};

use crate::{net::connection::Connection, server::submit::arm_multishot_accept};

pub fn handle_accept_cqe(ring: &mut IoUring, listen_fd: i32, res: i32, flags: u32) {
    
    let is_more = cqueue::more(flags);

    if res >= 0 {
        let client_fd = res;
        println!("Accept event: client_fd={client_fd}, more={is_more}");
        let _conn = Connection::new(client_fd);

        let rc = unsafe { libc::close(client_fd) };
        if rc < 0 {
            println!("Close client fd failed {}", io::Error::last_os_error());
        }
    }else {
        let errno = -res;
        println!("Accept Completion Error {}", errno);
    }

    if res < 0 || !is_more {
        println!("re-arming mutlishot accept");
        arm_multishot_accept(ring, listen_fd);
    }

}
