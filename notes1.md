## The Normal World 

Normally when your laptop talks to the internet 
-> you open a website, your app talks to the kernel’s networking stack, kernel passes packets to the network card (wifi / ethernet) -> out to the internet 
-> you never see the raw packets 

When your laptop sends a packet: 

Your app -> syscall (send()) -> Kernel TCP/IP stack -> driver -> network card -> Ethernet / Wifi wire 

We want to build our own networking stack instead of using the kernel's and we want to see and send raw packets ourselves

Linux gives us special fake networking cards called TUN / TAP: 
- TUN = IP packets (layer 3)
- TAP = ethernet frames (layer 2)

These are real interfaces as far as the kernel is concerned and the kernel does not treat them as fake 

`/dev/net/tun` is not a file -- it is a gateway into the kernel networking stack

We will use TAP because we want ethernet frames: 

When we read from TAP, we get raw ethernet frames: [ Destination MAC ][ Source MAC ][ Type ][ Payload ... ]
    - Destination MAC: who it’s for on the local network
    - Source MAC: who sent it 
    - Type = whats inside
    - Payload = the actual packet
 
With TAP, what we do is: 
    - Create a fake NIC (how computer connects to network) called tap0 
    - If we send data to tap0, the kernel delivers it to my program instead of a real wire 
    - If my program writes to tap0, the kernel thinks those bytes came from the network 


How it works: 
    - I create a tap interface (tap0) → fake network card 
    - To connect my program to that TAP device, I open /dev/net/tun 
    - You tell kernel that you want to bind this file to tap0 in tap mode 

This allows us to both give me ethernet frames arriving at tap0 and also writing to it injecting ethernet frames into tap0.

Each read gives you raw bytes: 

[ Ethernet Header ][ Payload... ]

Now we have to 

1) Parse the Ethernet header
2) Look at the ethertype 
3) Decide what protocol it is 
4) Dispatch it to the correct handler 

## Step 1: Parse Ethernet 

Ethernet frame layoutr: 

Bytes      Field
---------------------------
(0-5) -->  Dest MAC
(6-11) --> Src MAC
(12-13) --> EtherType 
(14+) -->  Payload

EtherType tells you what's inside 

Type      Meaning
---------------------------
0x0806 -->  ARP
0x0800 --> IPv4
0x86DD --> IPv6

The logic is then 

```
if ethertype == ARP → handle_arp()
if ethertype == IPv4 → handle_ip()
```

## Step 2: ARP: "Who has IP X?" 

ARP is how machines find each other's MAC addresses 

When the kernel wants to send to 10.0.0.1 it broadcasts "Who has 10.0.0.1"

and our fake NIC now receives this ARP request.

Our job is to say if the target_ip == my IP we reply with ```10.0.0.1 is at aa:bb:cc:dd:ee:ff```

## Step 3: IPv4 packets 

Once we have done ARP, we have to deal with IPV4. 

When our TAP program reads an Ethernet frame, ```[ Ethernet ][ IPv4 ][ TCP/UDP/... ]```

we first parse the IPv4 header, extract the destination IP, protocol, and payload offset and check if the IP's match.

Now, 

if protocol == 1  → ICMP  (ping)
if protocol == 6  → TCP
if protocol == 17 → UDP

Sample flow when someone calls ```ping 10,0.0.1```: 

1) Their kernel sends ethernet frame to tap0 
2) Our prgram reads it 
3) EtherType = IPv4 --> so we call the parse IP header 
4) Protocol = ICMP 
5) ICMP type = echo request
6) Build a reply 
7) Wrap in Ethernet 
8) Write back to tap0 
9) Kernel delivers it like it came from the network


Here is the mental model: 

Ethernet frame arrives →
Kernel hands it to tap0 →
Your FD receives it →
You interpret bytes →
You create new bytes →
write(fd) →
Kernel believes network sent them →
Remote machine receives reply 






