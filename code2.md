Right now, with our TAP implementation, we have the Ethernet frames which is the outer envelope with MAC addresses. 

Inside this envelope can be different payloads one of which is an ARP message. So our logic for this part of the code is something like: 

1) read frame --> ensure that len(ethernet frame header) >= 14 
2) parse ethernet header to get the ethertype 
3) ignore all non-ARP ethertypes 
4) Parse ARP: 
- Validate htype=1, ptype=0x0800, hlen=6, plen=4.
- Extract op, sha, spa, tha, tpa.

5) If `op==1` and `tpa = MY_IP`, build the response: 

```Ethernet: dst = sha, src = MY_MAC, type=0x0806

ARP: op=2, sha=MY_MAC, spa=MY_IP, tha=sha, tpa=spa
```

6) Write the 42-byte reply to the TAP fd.


So first we have the struct:

```
struct ArpV4 {
    op:  u16,     // the opera: 1=request, 2=reply
    sha: [u8;6],  // sender's MAC
    spa: [u8;4],  // sender's IP
    tha: [u8;6],  // target MAC
    tpa: [u8;4],  // target IP 
}
```

This is a Rust struct that matches the key fields payload. 

Then, `parse_arp_v4()` takes the slice after the Ethernet header and 

1) checks it's at least 28 bytes
2) validates the header constraints: 
Validates the header constants:
htype==1 (Ethernet)
ptype==0x0800 (IPv4)
hlen==6 (MAC length)
plen==4 (IPv4 length)

So we want to read the remaining fields into the ArpV4 structureci:

Reads the remaining fields into your ArpV4 str:

op (request or reply)
sha (sender's MAC)
spa (sender's IP)
tha (target MAC)
tpa (target IP)

It has a fixed sturcture  so we create a struct: 

```
struct Arpv4 { op: u16, sha: [u8;6], spa: [u8;4], tha: [u8;6], tpa: [u8;4] }
```

This is the - that describes the format of the ARP. 

