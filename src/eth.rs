#[derive(Clone, Copy)]
pub struct EthHdr {
    pub dst: [u8;6],
    pub src: [u8;6],
    pub etype: u16,
}

pub fn parse_eth(frame: &[u8]) -> Option<(EthHdr, &[u8])> {
    if frame.len() < 14 { return None; }
    Some((
        EthHdr {
            dst: frame[0..6].try_into().ok()?,
            src: frame[6..12].try_into().ok()?,
            etype: u16::from_be_bytes([frame[12], frame[13]])
        },
        &frame[14..]
    ))
}
