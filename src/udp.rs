pub fn parse_udp(p: &[u8]) -> Option<([u8;2], [u8;2], &[u8])> {
    if p.len() < 8 { return None; }
    Some((
        p[0..2].try_into().ok()?,
        p[2..4].try_into().ok()?,
        &p[8..]
    ))
}
