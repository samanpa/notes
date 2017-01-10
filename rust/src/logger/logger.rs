use core::time::Time;

#[repr(C)]
struct FileHeader
{
    magic : u64,
    msgHeaderSize : u16,
    version : u8
};

#[repr(C)]
struct MessageHeader
{
    time : Time,
    seqnum : u64,
    length : u32,
    msgType : u16
}
    
