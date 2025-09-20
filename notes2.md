Currently, our fd ↔ tap0 lets us read/write raw Ethernet frames. But right now they’re just blobs of bytes.

We now want our program to interpret those bytes — to parse the headers, recognize what kind of packet it is, and eventually act on it (replying to ARP).

## Ethernet frames (Layer 2)

Ethernet frame = smallest unit that my TAP sees 
Structure: 
-> 6 bytes: Destination MAC
-> 6 bytes: Source MAC 
-> 2 bytes: Ethertype 

The payload is what comes next (ARP, IPv4, IPv6)

Currently, in order for us to send data / IP packet from Host A to Host B, these are the steps: 

1) Host A wants to send an IP packet to B (10.0.0.1)
2) Host A must wrap that IP packet inside an ethernet frame 
3) But an ethernet frame requires a destination MAC and can't process just the direct IP (10.0.0.1)
4) A only knows B's IP not it's MAC. 
5) This is where ARP comes in -- A asks 'Who has 10.0.0.1'and B responds with its MAC address. 
6) Now A can send the IP packet / data / ethernet frames to B 


Every time we call read() from our file descriptor, we get one Ethernet frame so we need to peel off
the first 14 bytes to see what you are looking at.

## MAC Addresses 

Uniquely identifies a network interface card 
Unicast MAC: 
-> one to one communication
-> a frame is addressed to a specific MAC address 
-> only the NIC with that MAC will accept the frame 

Broadcast MAC: 
-> Special MAC: `ff:ff:ff:ff:ff:ff`
-> ALL NIC's on the local network will accept it 
-> Used for ARP when Host A doesn't know B's MAC, it will send an ARP request to broadcast 
-> Every NIC sees it but only the one that owns the IP replies 

FULL FLOW: 

1) Host A wants to send something to Host B 
    - Example: A wants to `ping 10.0.0.1`
    - A only knows B's IP address (`10.0.0.1`)
    - To actually put it on the LAN, A has to wrap the request inside an Ethernet frame 

2) Ethernet needs a destination MAC and can't just directly take in the IP: 
    - A has to know "What is the MAC address for IP 10.0.0.1" 

3) To do this, A sends an ARP request with a broadcast MAC. 
    - A doesn't know B's MAC so it uses ARP 
    - So it creates an ethernet frame: 
        `dstMac = ff:ff:ff:ff:ff:ff` (broadcast -> everyone on LAN hears it)
        `srcMac = A's own MAC`
        `ethertype = ARP` 
    - This makes a shout to the whole LAN asking who owns IP 10.0.0.
    
4) B responds sends back an ARP to respond
    - B builds a unicast frame back to A: 
        - dstMac = A's MAC 
        - srcMac = B's MAC
        - ethertype = ARP 
        - Payload = “IP 10.0.0.1 is at MAC 02:00:00:00:00:01.”
    - Only A accepts this response because it is directly addressed to A's MAC 

5) A updates ARP cache 

6) Now A can send real data (IP packets wrapped in Ethernet): 
    - A builds an ethernet frame with the destination MAC set to B's MAC 
    - The payload inside this ethernet frame contains the actual data / IP packet 
    - B then responds appropriately

