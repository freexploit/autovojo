#![no_std]
#![no_main]
#![allow(nonstandard_style, dead_code)]

use aya_bpf::{
    bindings::xdp_action,
    helpers::bpf_csum_diff,
    macros::{map, xdp},
    maps::{HashMap, PerfEventArray},
    programs::XdpContext,
};
use aya_log_ebpf::info;
//use heapless::Vec;

use autovojo_common::{PacketLog, Tunnel};
use core::mem;
//use memoffset::offset_of;

mod bindings;
use bindings::{ethhdr, iphdr, udphdr};

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}

#[map(name = "EVENTS")]
static mut EVENTS: PerfEventArray<PacketLog> =
    PerfEventArray::<PacketLog>::with_max_entries(1024, 0);

#[map(name = "TUNNELS")]
static mut TUNNELS: HashMap<u16, Tunnel> = HashMap::<u16, Tunnel>::with_max_entries(1024, 0);

#[xdp]
pub fn xdp_redirect(ctx: XdpContext) -> u32 {
    match try_xdp_redirect(ctx) {
        Ok(ret) => ret,
        Err(_) => xdp_action::XDP_ABORTED,
    }
}


#[inline(always)]
fn csum_fold_helper(mut csum: u32) -> u16 {
    csum = (csum & 0xffff) + (csum >> 16);
    csum = !((csum & 0xffff) + (csum >> 16));
    csum as u16
}

#[inline(always)]
fn ptr_at<T>(ctx: &XdpContext, offset: usize) -> Option<*const T> {
    let start = ctx.data();
    let end = ctx.data_end();
    let len = mem::size_of::<T>();

    if start + offset + len > end {
        return None;
    }

    Some((start + offset) as *const T)
}

#[inline(always)]
fn ptr_at_mut<T>(ctx: &XdpContext, offset: usize) -> Option<*mut T> {
    let ptr = ptr_at::<T>(ctx, offset)?;
    Some(ptr as *mut T)
}

fn try_xdp_redirect(ctx: XdpContext) -> Result<u32, u32> {
    let eth = ptr_at::<ethhdr>(&ctx, 0).ok_or(xdp_action::XDP_PASS)?;
    if unsafe { u16::from_be((*eth).h_proto) } != ETH_P_IP {
        return Ok(xdp_action::XDP_PASS);
    }

    let mut ip = ptr_at_mut::<iphdr>(&ctx, ETH_HDR_LEN).ok_or(xdp_action::XDP_PASS)?;

    if unsafe { (*ip).protocol } != IPPROTO_UDP {
        return Ok(xdp_action::XDP_PASS);
    }

    let mut udp =
        ptr_at_mut::<udphdr>(&ctx, ETH_HDR_LEN + IP_HDR_LEN).ok_or(xdp_action::XDP_PASS)?;

    info!(&ctx, "received a UDP packet");

    let tunnel = get_tunnel(unsafe { u16::from_be((*udp).dest) });

    let current_check = ! unsafe { (*udp).check };

    let action = if let Some(t) = tunnel {

        let old_port = unsafe {u32::to_be((*udp).dest.into())};
        let new_port = u32::to_be(t.dst_port.into());

        let new_check = unsafe {
            let old = &old_port as *const u32;
            let new = &new_port as *const u32;
            bpf_csum_diff(old as *mut u32,4, new as *mut u32,4, current_check as u32) as u32
        };
        info!(&ctx, "Got a packet on port 55555");

        let check = csum_fold_helper(new_check);
        unsafe {
            (*ip).daddr = u32::to_be(t.ipv4_address);
            (*udp).dest = u16::to_be(t.dst_port);
            (*udp).check = !check;
        };
        xdp_action::XDP_TX
    } else {
        xdp_action::XDP_PASS
    };

    let log_entry = PacketLog {
        ipv4_address: unsafe { u32::from_be((*ip).saddr) },
        action,
        src_port: unsafe { u16::from_be((*udp).source) },
        dst_port: unsafe { u16::from_be((*udp).dest) },
    };

    //info!(&ctx, "received a packet");
    unsafe {
        EVENTS.output(&ctx, &log_entry, 0); //
    }
    Ok(action)
}

fn get_tunnel(port: u16) -> Option<&'static Tunnel> {
    unsafe { TUNNELS.get(&port) }
}
const ETH_P_IP: u16 = 0x0800;
const IPPROTO_UDP: u8 = 0x0011;
const ETH_HDR_LEN: usize = mem::size_of::<ethhdr>();
const IP_HDR_LEN: usize = mem::size_of::<iphdr>();
