use nix::fcntl::{open, OFlag};
use nix::sys::stat::Mode;
use std::fs::File;
use std::mem::zeroed;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

pub fn open_dev_net_tun() -> std::io::Result<OwnedFd> {
    let fd = open("/dev/net/tun", OFlag::O_RDWR, Mode::empty())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    Ok(unsafe { OwnedFd::from_raw_fd(fd) })
}

pub fn attach_tap(fd: OwnedFd, ifname: &str) -> std::io::Result<File> {
    #[repr(C)]
    struct Ifreq { ifr_name: [u8;16], ifr_flags: i16, pad: [u8;24] }

    const IFF_TAP: i16 = 0x0002;
    const IFF_NO_PI: i16 = 0x1000;
    const TUNSETIFF: u64 = 0x400454ca;

    let mut ifr: Ifreq = unsafe { zeroed() };
    for (i,b) in ifname.as_bytes().iter().take(15).enumerate() {
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
