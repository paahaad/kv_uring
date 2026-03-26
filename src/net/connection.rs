use std::os::fd::RawFd;

use crate::protocal::ParserState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnState {
    Idle,       // connection exist but no IO in flight
    Receiving,  // a recv SQE is currently in flight
    Sending,    // a send SQE is currently in flight
    Closing,    // terminal state
}

#[derive(Debug)]
pub struct Connection {
    pub fd: RawFd,
    pub state: ConnState,
    pub read_buf_id: Option<u16>, // which buffer is currently assigned
    pub write_queue: Vec<Vec<u8>>, // TODO
    pub parser_state: crate::protocal::ParserState // TODO
}

impl Connection {
    pub fn new(fd: RawFd) -> Self {
        Self { 
            fd, 
            state: ConnState::Idle, 
            read_buf_id: None, 
            write_queue: Vec::new(), 
            parser_state: ParserState::default()
        }
    }
}