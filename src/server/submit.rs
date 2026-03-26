use io_uring::{IoUring, opcode, types};
use libc::*;

use crate::server::op_tags::{OpKind, pack_user_data};

pub fn arm_multishot_accept(ring: &mut IoUring, listen_fd: i32) {
    let accept_ud = pack_user_data(OpKind::Accept, 0, 0);
    let accept_sqe = opcode::AcceptMulti::new(types::Fd(listen_fd))
        .flags(SOCK_NONBLOCK | SOCK_CLOEXEC)
        .build()
        .user_data(accept_ud);

    {
        let mut sq = ring.submission();
        unsafe {
            sq.push(&accept_sqe).expect("SQ is full");
        }
    }
    ring.submit().expect("Submit multishot accept failed");

}
