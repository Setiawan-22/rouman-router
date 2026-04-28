#![no_std]
#![no_main]

use aya_ebpf::{
    macros::{map, xdp},
    maps::{HashMap, Array},
    programs::XdpContext,
};

use network_types::{
    eth::EthHdr,
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
    let start = ctx.data();
    let end = ctx.data_end();
    let mut action = xdp_action::XDP_PASS;

    // 1. Ethernet Header
    if start + EthHdr::LEN <= end {
        let ethhdr = start as *const EthHdr;
        if unsafe { u16::from_be((*ethhdr).ether_type) } == 0x0800 {
            // 2. IPv4 Header
            if start + EthHdr::LEN + Ipv4Hdr::LEN <= end {
                let ipv4hdr = (start + EthHdr::LEN) as *const Ipv4Hdr;
                let source_addr = u32::from_ne_bytes(unsafe { (*ipv4hdr).src_addr });

                if let Some(val) = STATS.get_ptr_mut(STAT_TOTAL) {
                    unsafe { *val += 1 };
                }

                if unsafe { BLACKLIST.get(&source_addr) }.is_some() {
                    if let Some(val) = STATS.get_ptr_mut(STAT_DROPPED) {
                        unsafe { *val += 1 };
                    }
                    action = xdp_action::XDP_DROP;
                } else {
                    // 3. TCP Check
                    if unsafe { (*ipv4hdr).proto } == IpProto::Tcp {
                        // Dynamic Offset for TLS based on IPv4 IHL and TCP Data Offset
                        let mut sni = [0u8; 32];
                        if extract_sni_dynamic(&ctx, &mut sni) == 1 {
                            if unsafe { SNI_BLACKLIST.get(&sni) }.is_some() {
                                if let Some(val) = STATS.get_ptr_mut(STAT_DROPPED) {
                                    unsafe { *val += 1 };
                                }
                                action = xdp_action::XDP_DROP;
                            }
                        }
                    }
                }
            }
        }
    }

    action
}

/// Dynamic SNI extraction that calculates exact IPv4 and TCP header lengths.
/// This prevents SNI extraction failures on packets with IPv4/TCP options.
fn extract_sni_dynamic(ctx: &XdpContext, res: &mut [u8; 32]) -> u32 {
    let start = ctx.data();
    let end = ctx.data_end();

    // 1. Check if Ethernet header fits
    if start + 14 > end { return 0; }
    
    // 2. Read IPv4 IHL (Internet Header Length)
    if start + 14 + 1 > end { return 0; }
    let ihl = (unsafe { *((start + 14) as *const u8) } & 0x0F) * 4;
    let ipv4_len = ihl as usize;
    
    // eBPF Verifier help: Bound the length
    if ipv4_len < 20 || ipv4_len > 60 { return 0; }

    // 3. Read TCP Data Offset
    let tcp_start = start + 14 + ipv4_len;
    if tcp_start + 13 > end { return 0; }
    let doff = (unsafe { *((tcp_start + 12) as *const u8) } >> 4) * 4;
    let tcp_len = doff as usize;
    
    // eBPF Verifier help: Bound the length
    if tcp_len < 20 || tcp_len > 60 { return 0; }

    // 4. Calculate TLS Payload Offset
    let tls_offset = 14 + ipv4_len + tcp_len;
    
    // 5. Check if it's a TLS Handshake (0x16)
    if start + tls_offset + 1 > end { return 0; }
    if unsafe { *((start + tls_offset) as *const u8) } == 0x16 {
        // Check for SNI at the dynamic offset
        let sni_start = tls_offset + 43;
        
        // Unrolled bounds-checked scanning
        if check_sni_at(start, end, sni_start, res) == 1 { return 1; }
        if check_sni_at(start, end, sni_start + 32, res) == 1 { return 1; }
        if check_sni_at(start, end, sni_start + 64, res) == 1 { return 1; }
        if check_sni_at(start, end, sni_start + 96, res) == 1 { return 1; }
    }

    0
}

#[inline(always)]
fn check_sni_at(start: usize, end: usize, off: usize, res: &mut [u8; 32]) -> u32 {
    // We scan a window of 32 bytes starting at 'off' for the SNI extension
    for i in 0..8 {
        let p = start + off + (i * 4);
        if p + 14 > end { break; }
        
        unsafe {
            if *(p as *const u8) == 0 && *((p + 1) as *const u8) == 0 {
                if *((p + 4) as *const u8) == 0 && *((p + 6) as *const u8) == 0 {
                    let nlen = (((*((p + 7) as *const u8) as usize) << 8) | (*((p + 8) as *const u8) as usize)) & 0xFFFF;
                    let name_p = p + 9;
                    if name_p + 32 <= end {
                        let copy = if nlen > 32 { 32 } else { nlen };
                        for j in 0..32 {
                            if j < copy {
                                res[j] = *((name_p + j) as *const u8);
                            }
                        }
                        return 1;
                    }
                }
            }
        }
    }
    0
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
