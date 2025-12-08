pub struct Ipv4Hdr<'a> {
    pub protocol: u8,
    pub src: [u8;4],
    pub dst: [u8;4],
    pub payload: &'a [u8],
}

pub fn parse_ipv4(buf: &[u8]) -> Option<Ipv4Hdr> {
    if buf.len() < 20 { return None; }
    let ihl = (buf[0] & 0x0F) as usize * 4;

    Some(Ipv4Hdr {
        protocol: buf[9],
        src: buf[12..16].try_into().ok()?,
        dst: buf[16..20].try_into().ok()?,
        payload: &buf[ihl..],
    })
}
