#![no_std]
#![no_main]
#![allow(nonstandard_style,dead_code)]

use heapless::Vec;
use aya_bpf::{
    bindings::xdp_action,
    macros::{map,xdp},
    maps::PerfEventArray,
    programs::XdpContext,
};
use aya_log_ebpf::info;

use core::mem;
use memoffset::offset_of;
use autovojo_common::PacketLog;

mod bindings;
use bindings::{ethhdr, iphdr};

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}

#[map(name = "EVENTS")]
static mut EVENTS: PerfEventArray<PacketLog> = PerfEventArray::<PacketLog>::with_max_entries(1024, 0);

//#[map(name = "NODES")]
//static mut NODES: PerfEventArray<PacketLog> = PerfEventArray::<PacketLog>::with_max_entries(1024, 0);

#[xdp]
pub fn xdp_firewall(ctx: XdpContext) -> u32 {
    match try_xdp_firewall(ctx) {
        Ok(ret) => ret,
        Err(_) => xdp_action::XDP_ABORTED,
    }
}

#[inline(always)] // 
unsafe fn ptr_at<T>(ctx: &XdpContext, offset: usize) -> Result<*const T, ()> {
    let start = ctx.data();
    let end = ctx.data_end();
    let len = mem::size_of::<T>();

    if start + offset + len > end {
        return Err(());
    }

    Ok((start + offset) as *const T)
}

fn try_xdp_firewall(ctx: XdpContext) -> Result<u32, ()> {

    let h_proto = u16::from_be(unsafe {
        *ptr_at(&ctx, offset_of!(ethhdr, h_proto))? // 
    });


    if h_proto != ETH_P_IP {
        return Ok(xdp_action::XDP_PASS);
    }

    let mut h_source: [u8;6] = unsafe {
        *ptr_at(&ctx, offset_of!(ethhdr,h_source))?
    };

    let le_h_source: Vec<u8,6> = h_source.iter_mut().map(|byte| u8::from_be(*byte)).collect();


    let source = u32::from_be(unsafe {
        *ptr_at(&ctx, ETH_HDR_LEN + offset_of!(iphdr, saddr))?
    });


    let mac: &[u8;6] = &[232,235,017,030,001,250];

    let action = if mac == &le_h_source.clone().into_array().unwrap() {
        xdp_action::XDP_PASS
    } else {
        xdp_action::XDP_DROP
    };


    let log_entry = PacketLog {
        ipv4_address: source,
        mac_address: le_h_source.into_array().unwrap(),
        padding: 0,
        action,
    };

    info!(&ctx, "received a packet");
    unsafe {
        EVENTS.output(&ctx, &log_entry, 0); // 
    }
    Ok(action)
}

const ETH_P_IP: u16 = 0x0800;
const ETH_HDR_LEN: usize = mem::size_of::<ethhdr>();
