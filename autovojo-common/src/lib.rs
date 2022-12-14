#![no_std]

pub type MacAddress = [u8; 6];

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Tunnel {
    pub ipv4_address: u32,
    pub dst_port: u16,
    pub src_port: u16,
    pub src_ipv4_address: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct PacketLog {
    pub ipv4_address: u32,
    pub src_port: u16,
    pub dst_port: u16,
    pub action: u32,
}

#[cfg(feature = "user")]
unsafe impl aya::Pod for PacketLog {}

#[cfg(feature = "user")]
unsafe impl aya::Pod for Tunnel {}
