A high-performance async key-value store server in Rust using io_uring as the sole async runtime — no tokio, no epoll, no blocking syscalls

- all network I/O via io_uring (IORING_OP_ACCEPT, IORING_OP_RECV, IORING_OP_SEND, multishot where applicable)
- All disk I/O via io_uring (IORING_OP_READ, IORING_OP_WRITE for WAL persistence)
- Fixed buffers (IORING_REGISTER_BUFFERS) for zero-copy paths
- Single-threaded event loop with a completion queue dispatch table
- Redis RESP protocol (subset: GET, SET, DEL, PING)
- Append-only WAL for persistence, with startup replay
- No tokio, no async-std — use io-uring crate (tokio-uring is off limits)