mod tun;
mod eth;
mod arp;
mod ipv4;
mod udp;
mod tcp;

use tun::*;
use eth::*;
use arp::*;
use ipv4::*;
use udp::*;
use tcp::*;

use std::io::{Read, Write};
use std::time::{Duration, Instant};
use std::collections::VecDeque;

pub const MY_MAC: [u8;6] = [0x02,0x00,0x00,0x00,0x00,0x01];
pub const MY_IP:  [u8;4] = [10,0,0,1];

fn main() -> std::io::Result<()> {
    let fd = open_dev_net_tun()?;
    let mut file = attach_tap(fd, "tap0")?;

    let mut tcp = TcpConn {
        state: TcpState::Listen,
        seq: 100,
        ack: 0,
        send_buf: VecDeque::new(),
        recv_buf: VecDeque::new(),
        last_tx: Instant::now(),
    };

    let mut buf = [0u8;2048];
    let mut out = Vec::new();

    loop {
        let n = file.read(&mut buf)?;
        if let Some((eth, payload)) = parse_eth(&buf[..n]) {
            match eth.etype {
                0x0806 => { // ARP
                    if let Some(arp) = parse_arp(payload) {
                        if arp.op == 1 && arp.tpa == MY_IP {
                            build_arp_reply(&mut out, &eth, &arp, MY_MAC, MY_IP);
                            file.write_all(&out)?;
                        }
                    }
                }

                0x0800 => { // IPv4
                    if let Some(ip) = parse_ipv4(payload) {
                        match ip.protocol {
                            17 => { let _ = parse_udp(ip.payload); }
                            6 => { // TCP
                                if let Some((_sp,_dp,seq,_ack,flags,_data)) = parse_tcp(ip.payload) {
                                    if flags & 0x02 != 0 && tcp.state == TcpState::Listen {
                                        tcp.state = TcpState::SynRecv;
                                        tcp.ack = seq + 1;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        if tcp.send_buf.len() > 0 && tcp.last_tx.elapsed() > Duration::from_millis(500) {
            tcp.last_tx = Instant::now();
        }
    }
}
