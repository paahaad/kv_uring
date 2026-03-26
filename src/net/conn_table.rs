use slab::Slab;

use crate::net::connection::Connection;

pub type ConnId = u16;

#[derive(Debug)]
pub struct ConnectionTable {
    conns: Slab<Connection>,
}

impl ConnectionTable {
    pub fn new() -> Self {
        Self {
            conns: Slab::with_capacity(1024) // will tune later
        }
    }

    pub fn insert(&mut self, conn: Connection) -> ConnId {
        
        let conn_id = self.conns.insert(conn);
        if conn_id > u16::MAX as usize {
            panic!("Slab Overflow");
        }
        conn_id as ConnId
    }

    pub fn get(&self, conn_id: ConnId) -> Option<&Connection> {
        self.conns.get(conn_id as usize)
    }
    pub fn get_mut(&mut self, conn_id: ConnId) -> Option<&mut Connection> {
        self.conns.get_mut(conn_id as usize)
    }

    pub fn remove(&mut self, conn_id: ConnId) -> Option<Connection> {
        if self.conns.contains(conn_id as usize) {
            Some(self.conns.remove(conn_id as usize))
        }else {
            None
        }
    }
}


// without a centralized close path we can Double-close fd, leak slab/ buffer
pub fn close_connection(conn_id: ConnId, conn_table: &mut ConnectionTable) {
    if let Some(conn) = conn_table.remove(conn_id) {
        unsafe {
            libc::close(conn.fd);
        }
        println!("Closed: fd={}, conn_id={}", conn.fd, conn_id)
        // TODO: return buffer to pool
    } else {
        println!("close_connections: conn_id={} already removed", conn_id);
    }
}
