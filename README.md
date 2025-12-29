# Mini Internet Stack (Rust)

A minimal user-space implementation of core Internet networking protocols built from scratch in Rust, integrated with Linux using TUN/TAP virtual network interfaces. This project implements packet parsing, protocol handling, and buffering.

## Features

- **Ethernet**
    - Frame parsing and construction
- **ARP (IPv4)**
    - ARP request parsing
    - ARP reply generation for local IP
- **IPv4**
    - Header parsing and payload extraction
- **UDP**
    - Datagram parsing
- **TCP (Basic)**
    - TCP header parsing
    - Sequence and acknowledgment tracking
    - Send/receive buffering
    - Basic retransmission logic
- **Kernel Integration**
    - Real packet transmission and reception using Linux **TUN/TAP**
- **Debugging & Validation**
    - Packet inspection using `tcpdump` and `Wireshark`

## Architecture

Application -> TCP / UDP -> IPv4 -> ARP / Ethernet -> TUN/TAP → Linux Kernel → Network Interface

Each protocol layer is implemented in its own Rust module.

## Project Structure

src/
    main.rs # Packet loop and protocol dispatch
    tun.rs # TUN/TAP device setup and ioctl handling
    eth.rs # Ethernet frame parsing
    arp.rs # ARP parsing and reply generation
    ipv4.rs # IPv4 header parsing
    udp.rs # UDP datagram parsing
    tcp.rs # Basic TCP state machine and buffering

## How It Works

1. Attaches to a Linux TAP interface (`tap0`)
2. Reads raw Ethernet frames from the kernel
3. Parses protocol layers (Ethernet → ARP / IPv4 → UDP / TCP)
4. Handles:
    - ARP requests for the local IP
    - TCP SYN packets and connection state
5. Writes frames back to the TAP interface for real network transmission

## Requirements

- Linux
- Rust (stable)
- Root permissions (required for TUN/TAP)

### Dependencies

- nix
- libc

## Running

Create and configure TAP device:

1) ```sudo ip tuntap add dev tap0 mode tap```
2) ```sudo ip link set tap0 up```
3) ```sudo ip addr add 10.0.0.1/24 dev tap0```

Run the stack:

```sudo cargo run```

You can test ARP, ICMP, and TCP traffic using:

1) ```ping 10.0.0.1```
2) ```tcpdump -i tap0```    