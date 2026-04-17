#![no_std]
#![no_main]

use aya_ebpf::{
    macros::{map, xdp},
    maps::{HashMap, Array},
    programs::XdpContext,
};

use core::mem;
use network_types::{
    eth::{EthHdr, EtherType},
    ip::{Ipv4Hdr, IpProto},
    tcp::TcpHdr,
};

#[map]
pub static BLACKLIST: HashMap<u32, u32> = HashMap::with_max_entries(1024, 0);

#[map]
pub static SNI_BLACKLIST: HashMap<[u8; 32], u32> = HashMap::with_max_entries(1024, 0);

#[map]
pub static STATS: Array<u64> = Array::with_max_entries(2, 0);

// Indeks Statistik
const STAT_TOTAL: u32 = 0;
const STAT_DROPPED: u32 = 1;

#[xdp]
pub fn rouman_firewall(ctx: XdpContext) -> u32 {
    match try_rouman_firewall(&ctx) {
        Ok(ret) => ret,
        Err(_) => xdp_action::XDP_ABORTED,
    }
}

fn try_rouman_firewall(ctx: &XdpContext) -> Result<u32, u32> {
    let ethhdr: *const EthHdr = unsafe { ptr_at(ctx, 0)? };
    match u16::from_be(unsafe { (*ethhdr).ether_type }) {
        0x0800 => {} // IPv4
        _ => return Ok(xdp_action::XDP_PASS),
    }

    let ipv4hdr: *const Ipv4Hdr = unsafe { ptr_at(ctx, EthHdr::LEN)? };
    let source_addr = u32::from_ne_bytes(unsafe { (*ipv4hdr).src_addr });

    // Update Statistik Total
    if let Some(val) = STATS.get_ptr_mut(STAT_TOTAL) {
        unsafe { *val += 1 };
    }

    // Cek Blacklist
    if unsafe { BLACKLIST.get(&source_addr) }.is_some() {
        if let Some(val) = STATS.get_ptr_mut(STAT_DROPPED) {
            unsafe { *val += 1 };
        }
        return Ok(xdp_action::XDP_DROP);
    }

    Ok(xdp_action::XDP_PASS)
}

#[inline(always)]
unsafe fn ptr_at<T>(ctx: &XdpContext, offset: usize) -> Result<*const T, u32> {
    let start = ctx.data();
    let end = ctx.data_end();
    let len = mem::size_of::<T>();

    if start + offset + len > end {
        return Err(xdp_action::XDP_ABORTED);
    }

    Ok((start + offset) as *const T)
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    unsafe { core::hint::unreachable_unchecked() }
}

pub mod xdp_action {
    pub const XDP_ABORTED: u32 = 0;
    pub const XDP_DROP: u32 = 1;
    pub const XDP_PASS: u32 = 2;
    pub const XDP_TX: u32 = 3;
    pub const XDP_REDIRECT: u32 = 4;
}
