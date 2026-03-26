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
