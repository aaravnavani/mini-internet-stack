use nix::fcntl::{open, OFlag};
use nix::sys::stat::Mode;
use std::fs::File;
use std::mem::zeroed;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

//end goal: file descriptor to behave like a network card (NIC)

// open /dev/net/tun -- this allows us to actually create TUN/TAP interfaces 
// (opens the kernel's TUN/TAP control device), ask for read/write (`OFlag::O_RDWR`) because we want to both read packets arriving and write packets back 
// This is all wrapped under a rust file descriptor 
pub fn open_dev_net_tun() -> std::io::Result<OwnedFd> {
    let fd = open("/dev/net/tun", OFlag::O_RDWR, Mode::empty())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    Ok(unsafe { OwnedFd::from_raw_fd(fd) })
}

// connect this file descriptor to a TAP interface (tap0)
// 
pub fn attach_tap(fd: OwnedFd, ifname: &str) -> std::io::Result<File> {
    #[repr(C)]
    struct Ifreq { ifr_name: [u8;16], ifr_flags: i16, pad: [u8;24] }

    const IFF_TAP: i16 = 0x0002;
    const IFF_NO_PI: i16 = 0x1000;
    const TUNSETIFF: u64 = 0x400454ca;

    let mut ifr: Ifreq = unsafe { zeroeed() };
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

//now if kernel wants to send a packet to us, it will write to our file descriptor
// if we write to the file descriptor, kernel thinks those bytes came from the network