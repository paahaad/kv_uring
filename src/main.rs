use libc::*;
use std::{io, mem};
use io_uring::{IoUring, cqueue, opcode, types};

/*
This is Op tagging, meaning we need to encode the metadata about the asyc operation into
single identifier, in this case u64. so that we can quicky determine what kind of operation
it was and which resources its related to

Why we need op tagging?
When we fire many asyc operation simultaneously, the completions come back out of order. lets 
say a CQE arrives but which operation is that, a read, a write or accept?

The kernal give us exactly one thing to identify it: the `user_data` a raw u64 which we set 
when submiting thr SQE. The kernal copies the exact same 64-bit value into CQE. thats our only 
correlation handle.

so we pack everything we need to dispatch the completions into one u64.

## BIT LAYOUT

A u64 is 64 bit, and split like this

63       48 47      32 31        16 15         0
 ┌──────────┬──────────┬────────────┬────────────┐
 │  (spare) │  op_kind │   conn_id  │   buf_id   │
 │  16 bits │  16 bits │   16 bits  │   16 bits  │
 └──────────┴──────────┴────────────┴────────────┘

*/


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u16)]
pub enum OpKind{
    Accept  = 1, 
    Recv    = 2,
    Send    = 3, 
    Close   = 4
}


impl OpKind {
    fn from_u16(v: u16) -> Option<Self> {
        match v {
            1 => Some(Self::Accept),
            2 => Some(Self::Recv),
            3 => Some(Self::Send),
            4 => Some(Self::Close),
            _ => None,
        }
    }
}

#[inline]
pub fn pack_user_data(op: OpKind, conn_id: u16, buf_id: u16) -> u64 {
    (op as u64) << 32 | (conn_id as u64) << 16 | buf_id as u64
}

#[inline]
pub fn unpack_user_data(v: u64) -> (OpKind, u16, u16) {
    let op_raw = ((v >> 32) & 0xFFFF) as u16;
    let conn_id = ((v >> 16) & 0xFFFF) as u16;
    let buf_id = ( v & 0xFFFF) as u16;

    let op = OpKind::from_u16(op_raw).unwrap_or_else(||panic!("Unknow OpKind {}, user_data - {:018x}", op_raw, v));
    (op, conn_id,  buf_id)
} 

fn arm_multishot_accept(ring: &mut IoUring, listen_fd: i32) {
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


fn handle_accept_cqe(ring: &mut IoUring, listen_fd: i32, res: i32, flags: u32) {
    
    let is_more = cqueue::more(flags);

    if res >= 0 {
        let client_fd = res;
        println!("Accept event: client_fd={client_fd}, more={is_more}");

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
        println!("Listening on 0.0.0.0:6379 (fd = {})", fd);

        // ring with queue depth 256, must pe power of two
        // this means that SQ can hold 256 in flight submition at once
        // CQ will be automtically SQx2 = 512

        let mut ring_fd: IoUring = IoUring::builder()
            .build(256)
            .expect("io_uring setup failed");

        println!("io_uring created (sq_depth=256, cq_depth={})", ring_fd.params().cq_entries());

        // Submit one Multishot accept SQE
        arm_multishot_accept(&mut ring_fd, fd);

        loop {
            ring_fd.submit_and_wait(1).expect("Submit_and_wait Failed");

            let mut queue: Vec<(u64, i32, u32)>= Vec::new();

            // ring_fd.completions() is borrowed mutable here thus scope {} ;
            {
                let cq =  ring_fd.completion();
                for cqe in cq {
                    queue.push((cqe.user_data(), cqe.result(), cqe.flags()));
                }
            }

            for (user_data, res, flags) in queue {
                let (op, conn_id, buf_id) = unpack_user_data(user_data);

                match op {
                    OpKind::Accept => handle_accept_cqe(&mut ring_fd, fd, res, flags),
                    _ => {
                        println!("Not supported Op Kind");
                    }
                }

            }
        }



    }
}
