```
// ----- open /dev/net/tun ----- fn open_dev_net_tun() -> std::io::Result<OwnedFd> { let fd = open("/dev/net/tun", OFlag::O_RDWR, Mode::empty()) .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?; Ok(unsafe { OwnedFd::from_raw_fd(fd) }) }
```

- Opens `/dev/net/tun` -- this allows us to actually create TUN/TAP interfaces 
- We ask for read/write (`OFlag::O_RDWR`) because we want to both read packets arriving and write packets back 
- This is all wrapped under a rust file descriptor 

This function opens the kernel's TUN/TAP control device

But we still need to connect this file descriptor to a TAP interface: 

```
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
```

We've opened `/dev/net/tun`, which essentially acts as a doorway into the kernel's actual virtual NIC. Now we have to tell the kernel what kind of virtual NIC and what name to bind it to (`tap0` in this case). this config is then passed through a system call called `ioctl`. `ioctl` takes: 

1) a file descriptor (which is what we just opened in the first step)
2) a pointer to a C struct containing the parameters desribed above

this is how we create the C struct: 

```
#[repr(C)]
struct Ifreq {
    ifr_name: [u8; 16], // IFNAMSIZ
    ifr_flags: i16,
    pad: [u8; 24],
}
```

`ifr_naem`: name of the interface we want (tap0)
`ifr_flags`: options/flags (TAP vs TUN, with/without metadata)
`pad`: filler bytes

We then set some flags: 

```
const IFF_TAP: i16   = 0x0002;   // TAP = give me Ethernet frames
const IFF_NO_PI: i16 = 0x1000;   // NO_PI = don’t prepend extra metadata
const TUNSETIFF: u64 = 0x400454ca; // ioctl request number
```

Then, 

```
let mut ifr: Ifreq = unsafe { zeroed() };
for (i, b) in ifname.as_bytes().iter().take(15).enumerate() {
    ifr.ifr_name[i] = *b;
}
ifr.ifr_flags = IFF_TAP | IFF_NO_PI;
```

This fills out the struct we just created with the flags we just set. 

```
let ret = unsafe { libc::ioctl(fd.as_raw_fd(), TUNSETIFF as _, &ifr) };
```

This is the actual call to `ioctl` passing in 

`fd`: handle to `/dev/net/tun` you opened earlier 
`ioctl`: issue device-specific commands 

With this command, we tell the kernel "on this file descriptor, attach to a TAP device named tap0 with certain flags "

Now, `fd` is bound to `tap0`


```let file = unsafe { File::from_raw_fd(fd.as_raw_fd()) };
std::mem::forget(fd);
Ok(file)
```

Now, we convert fd into a File so that we can call `file.read()` (ethernet frame arriving at tap0) and `file.write()` (we're injecting a raw ethernet frame out of tap0) to actually interact with tap0. 



