use nix::fcntl::{open, OFlag};
use nix::sys::stat::Mode;
use std::fs::File;
use std::io::Read;
use std::mem::zeroed;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

fn main() -> std::io::Result<()> {
    // 1) open the special device (the “doorway”)
    let fd = open_dev_net_tun()?;
    // 2) bind this fd to TAP mode on interface "tap0"
    let file = attach_tap(fd, "tap0")?;
    // 3) read raw Ethernet frames and print first few bytes
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
    // Minimal ifreq with name + flags (fits what TUNSETIFF needs)
    #[repr(C)]
    struct Ifreq {
        ifr_name: [u8; 16], // IFNAMSIZ
        ifr_flags: i16,
        pad: [u8; 24],      // pad so struct is large enough for ioctl path
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

    // Turn the same fd into a File for Read
    let file = unsafe { File::from_raw_fd(fd.as_raw_fd()) };
    std::mem::forget(fd); // File now owns it
    Ok(file)
}

// ----- read loop -----
fn read_frames(mut file: File) -> std::io::Result<()> {
    let mut buf = [0u8; 2048];

    loop {
        let n = file.read(&mut buf)?;
        if n == 0 { continue; }

        print!("{:>4} bytes: ", n);
        for b in &buf[..n.min(64)] { // print up to first 64 bytes
            print!("{:02x} ", b);
        }
        println!();
    }
}
