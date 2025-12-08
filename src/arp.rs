use crate::eth::EthHdr;

pub struct ArpV4 {
    pub op: u16,
    pub sha: [u8;6],
    pub spa: [u8;4],
    pub tha: [u8;6],
    pub tpa: [u8;4],
}

pub fn parse_arp(p: &[u8]) -> Option<ArpV4> {
    if p.len() < 28 { return None; }
    Some(ArpV4 {
        op:  u16::from_be_bytes([p[6], p[7]]),
        sha: p[8..14].try_into().ok()?,
        spa: p[14..18].try_into().ok()?,
        tha: p[18..24].try_into().ok()?,
        tpa: p[24..28].try_into().ok()?,
    })
}

pub fn build_arp_reply(
    out: &mut Vec<u8>,
    req_eth: &EthHdr,
    req: &ArpV4,
    my_mac: [u8;6],
    my_ip:  [u8;4]
) {
    out.clear();

    out.extend_from_slice(&req_eth.src);
    out.extend_from_slice(&my_mac);
    out.extend_from_slice(&0x0806u16.to_be_bytes());

    out.extend_from_slice(&1u16.to_be_bytes());
    out.extend_from_slice(&0x0800u16.to_be_bytes());
    out.push(6); out.push(4);

    out.extend_from_slice(&2u16.to_be_bytes());
    out.extend_from_slice(&my_mac);
    out.extend_from_slice(&my_ip);
    out.extend_from_slice(&req.sha);
    out.extend_from_slice(&req.spa);
}
