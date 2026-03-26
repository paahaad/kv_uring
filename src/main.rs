use io_uring::IoUring;

use crate::net::conn_table::ConnectionTable;

pub mod engine;
pub mod error;
pub mod io_uring_runtime;
pub mod net;
pub mod protocal;
pub mod server;
pub mod wal;

fn main() {
    let fd = net::listener::create_listener();
    println!("Listening on 0.0.0.0:6379 (fd = {})", fd);

    // ring with queue depth 256, must pe power of two
    // this means that SQ can hold 256 in flight submition at once
    // CQ will be automtically SQx2 = 512
    let mut ring_fd: IoUring = IoUring::builder()
        .build(256)
        .expect("io_uring setup failed");

    println!("io_uring created (sq_depth=256, cq_depth={})", ring_fd.params().cq_entries());

    server::run_event_loop(&mut ring_fd, fd);


}
