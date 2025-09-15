## The Normal World 

Normally when your laptop talks to the internet 
-> you open a website, your app talks to the kernel’s networking stack, kernel passes packets to the network card (wifi / ethernet) -> out to the internet 
-> you never see the raw packets 

We want to build our own networking stack instead of using the kernel's and we want to see and send raw packets ourselves

Linux gives us special fake networking cards called TUN / TAP: 
- TUN = IP packets (layer 3)
- TAP = ethernet frames (layer 2)

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


