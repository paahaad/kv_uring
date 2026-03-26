use io_uring::IoUring;

use crate::net::conn_table::ConnectionTable;
use crate::server::dispatch::handle_accept_cqe;
use crate::server::op_tags::{OpKind, unpack_user_data};
use crate::server::submit::arm_multishot_accept;

pub fn run_event_loop(ring_fd: &mut IoUring, listen_fd: i32) {
    // Submit one Multishot accept SQE
    arm_multishot_accept(ring_fd, listen_fd);
    let mut conn_table = ConnectionTable::new();


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
            let (op, _conn_id, _buf_id) = unpack_user_data(user_data);

            match op {
                OpKind::Accept => handle_accept_cqe(ring_fd, listen_fd, res, flags, &mut conn_table),
                _ => {
                    println!("Not supported Op Kind");
                }
            }
        }
    }
}
