use nix::fcntl::{open, OFlag};
use nix::sys::stat::Mode;
use std::fs::File;
use std::io::{Read, Write};
use std::mem::zeroed;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

const MY_MAC: [u8; 6] = [0x02, 0x00, 0x00, 0x00, 0x00, 0x01];
const MY_IP:  [u8; 4] = [10, 0, 0, 1];

fn main() -> std::io::Result<()> {
    let fd = open_dev_net_tun()?;
    let file = attach_tap(fd, "tap0")?;
    read_frames(file)
}

// ----- open /dev/net/tun -----
fn open_dev_net_tun() -> std::io::Result<OwnedFd> {
    let fd = open("/dev/net/tun", OFlag::O_RDWR, Mode::empty())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    Ok(unsafe { OwnedFd::from_raw_fd(fd) })
}

// ----- ioctl(TUNSETIFF) to get TAP on "tap0" -----
fn attach_tap(fd: OwnedFd, ifname: &str) -> std::io::Result<File> {
    #[repr(C)]
    struct Ifreq {
        ifr_name: [u8; 16], // IFNAMSIZ
        ifr_flags: i16,
        pad: [u8; 24],
    }
    const IFF_TAP: i16   = 0x0002;
    const IFF_NO_PI: i16 = 0x1000;
    const TUNSETIFF: u64 = 0x400454ca;

    let mut ifr: Ifreq = unsafe { zeroed() };
    for (i, b) in ifname.as_bytes().iter().take(15).enumerate() {
        ifr.ifr_name[i] = *b;
    }
    ifr.ifr_flags = IFF_TAP | IFF_NO_PI;

    let ret = unsafe { libc::ioctl(fd.as_raw_fd(), TUNSETIFF as _, &ifr) };
    if ret < 0 {
        return Err(std::io::Error::last_os_error());
    }
    let file = unsafe { File::from_raw_fd(fd.as_raw_fd()) };
    std::mem::forget(fd);
    Ok(file)
}

// ---------------------- Week 1: Ethernet + ARP ----------------------

#[derive(Clone, Copy)]
struct EthHdr { dst: [u8;6], src: [u8;6], etype: u16 }

fn parse_eth(frame: &[u8]) -> Option<(EthHdr, &[u8])> {
    if frame.len() < 14 { return None; }
    let hdr = EthHdr{
        dst: frame[0..6].try_into().ok()?,
        src: frame[6..12].try_into().ok()?,
        etype: u16::from_be_bytes([frame[12], frame[13]]),
    };
    Some((hdr, &frame[14..]))
}

struct ArpV4 { op: u16, sha: [u8;6], spa: [u8;4], tha: [u8;6], tpa: [u8;4] }

fn parse_arp_v4(p: &[u8]) -> Option<ArpV4> {
    if p.len() < 28 { return None; }
    let htype = u16::from_be_bytes([p[0], p[1]]);
    let ptype = u16::from_be_bytes([p[2], p[3]]);
    if !(htype == 1 && ptype == 0x0800 && p[4] == 6 && p[5] == 4) { return None; }
    Some(ArpV4{
        op:  u16::from_be_bytes([p[6], p[7]]),
        sha: p[8..14].try_into().ok()?,
        spa: p[14..18].try_into().ok()?,
        tha: p[18..24].try_into().ok()?,
        tpa: p[24..28].try_into().ok()?,
    })
}

fn build_arp_reply(out: &mut Vec<u8>, req_eth: &EthHdr, req: &ArpV4) {
    out.clear();

    // Ethernet header
    out.extend_from_slice(&req_eth.src);               // dst = requester MAC
    out.extend_from_slice(&MY_MAC);                    // src = our MAC
    out.extend_from_slice(&0x0806u16.to_be_bytes());   // type = ARP

    // ARP payload (Ethernet/IPv4 = 28 bytes)
    out.extend_from_slice(&1u16.to_be_bytes());        // htype = 1 (Ethernet)
    out.extend_from_slice(&0x0800u16.to_be_bytes());   // ptype = IPv4
    out.push(6); out.push(4);                          // hlen=6, plen=4
    out.extend_from_slice(&2u16.to_be_bytes());        // op = 2 (reply)
    out.extend_from_slice(&MY_MAC);                    // sha = our MAC
    out.extend_from_slice(&MY_IP);                     // spa = our IP
    out.extend_from_slice(&req.sha);                   // tha = requester MAC
    out.extend_from_slice(&req.spa);                   // tpa = requester IP
}

// ----- read loop -----
fn read_frames(mut file: File) -> std::io::Result<()> {
    let mut buf = [0u8; 2048];
    let mut out = Vec::with_capacity(64);

    loop {
        let n = file.read(&mut buf)?;
        if n < 14 { continue; } // must at least have Ethernet header

        // 1) Parse Ethernet
        if let Some((eth, payload)) = parse_eth(&buf[..n]) {
            // 2) Branch on Ethertype
            match eth.etype {
                0x0806 => { // ARP
                    if let Some(arp) = parse_arp_v4(payload) {
                        // 3) Act only on ARP request for our IP
                        if arp.op == 1 && arp.tpa == MY_IP {
                            build_arp_reply(&mut out, &eth, &arp);
                            file.write_all(&out)?;
                            // Optional: debug line
                            // eprintln!("replied: {} is at {:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                            //     format!("{}.{}.{}.{}", MY_IP[0],MY_IP[1],MY_IP[2],MY_IP[3]),
                            //     MY_MAC[0],MY_MAC[1],MY_MAC[2],MY_MAC[3],MY_MAC[4],MY_MAC[5]);
                        }
                    }
                }
                _ => { /* ignore non-ARP in Week 1 */ }
            }
        }
    }
}
