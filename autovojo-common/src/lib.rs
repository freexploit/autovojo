#![no_std]

pub type MacAddress = [u8;6];

#[repr(C)]
#[derive(Clone,Copy)]
pub struct PacketLog {
    pub ipv4_address: u32,
    pub action: u32,
    pub padding: u8,
    pub mac_address: MacAddress
}

#[cfg(feature="user")]
unsafe impl aya::Pod for PacketLog {}
