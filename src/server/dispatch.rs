use io_uring::{IoUring, cqueue};

use crate::net::{
    conn_table::ConnectionTable,
    connection::Connection
};
use crate::server::submit::arm_multishot_accept;


pub fn handle_accept_cqe(ring: &mut IoUring, listen_fd: i32, res: i32, flags: u32, conn_table: &mut ConnectionTable) {
    
    let is_more = cqueue::more(flags);

    if res >= 0 {
        let client_fd = res;
        let conn: Connection = Connection::new(client_fd);
        let conn_id = conn_table.insert(conn);
        println!("Accepted: fd={}, conn_id={}", client_fd, conn_id);

        println!("Connection Table: {:#?}", conn_table);
        
    }else {
        let errno = -res;
        println!("Accept Completion Error {}", errno);
    }

    if res < 0 || !is_more {
        println!("re-arming mutlishot accept");
        arm_multishot_accept(ring, listen_fd);
    }

}
