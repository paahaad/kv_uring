## What is IO_URING and why it exist

In Linux IO (read/write/accept) uses syscalls. Every syscall crosses user-kernal boundary making it expensive beacuse the CPU has to save registers, switch privilege level, run kernal code, then switch back. For high throughput server is doing thousands of accepts/reads/writes per second which is kind of bottleneck.

io_uring (introduced in linux 5.1) give you a shared memory ring between userspace and kernal. you submit the I/O operations by writing into ring buffer, and the kernal picks them up - often without a syscall at all. Result land in another ring buffer you poll. one syscall `io_uring_enter` can submit and reap hundreds of operations.

## The Two Ring

io_uring has two ring buffer. both in shared memory mapped between your process and kernal:

Your process                         Kernel
─────────────────────────────────────────────
SQ (Submission Queue)  ──push──▶  kernel reads SQEs, does I/O
CQ (Completion Queue)  ◀──push──  kernel writes CQEs when done

SQE = Submission Queue Entry i.e "Please do this I/O ops"
CQE = Completion Queue Entry i.e "here's the result of that operation"

