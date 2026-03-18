use super::{Encodable, RlpStream};

impl Encodable for bool {
    fn rlp_append(&self, s: &mut RlpStream) {
        if *self {
            s.append_raw(&[0x01]);
        } else {
            s.append_raw(&[0x80]);
        }
    }
}

impl Encodable for u64 {
    fn rlp_append(&self, s: &mut RlpStream) {
        if *self == 0 {
            s.append_raw(&[0x80]);
        } else if *self < 0x80 {
            s.append_raw(&[*self as u8]);
        } else {
            let bytes = to_big_endian_u64(*self);
            let encoded = encode_bytes(&bytes);
            s.append_raw(&encoded);
        }
    }
}

impl Encodable for Vec<u8> {
    fn rlp_append(&self, s: &mut RlpStream) {
        let encoded = encode_bytes(self);
        s.append_raw(&encoded);
    }
}

impl Encodable for String {
    fn rlp_append(&self, s: &mut RlpStream) {
        let encoded = encode_bytes(self.as_bytes());
        s.append_raw(&encoded);
    }
}

impl Encodable for &str {
    fn rlp_append(&self, s: &mut RlpStream) {
        let encoded = encode_bytes(self.as_bytes());
        s.append_raw(&encoded);
    }
}

pub(super) fn encode_bytes(data: &[u8]) -> Vec<u8> {
    if data.len() == 1 && data[0] < 0x80 {
        vec![data[0]]
    } else if data.len() <= 55 {
        let mut out = vec![0x80 + data.len() as u8];
        out.extend_from_slice(data);
        out
    } else {
        let len_bytes = to_big_endian_usize(data.len());
        let mut out = vec![0xb7 + len_bytes.len() as u8];
        out.extend_from_slice(&len_bytes);
        out.extend_from_slice(data);
        out
    }
}

pub(super) fn encode_list_header(payload_len: usize) -> Vec<u8> {
    if payload_len <= 55 {
        vec![0xc0 + payload_len as u8]
    } else {
        let len_bytes = to_big_endian_usize(payload_len);
        let mut out = vec![0xf7 + len_bytes.len() as u8];
        out.extend_from_slice(&len_bytes);
        out
    }
}

pub(super) fn to_big_endian_u64(value: u64) -> Vec<u8> {
    let bytes = value.to_be_bytes();
    let first_nonzero = bytes.iter().position(|&b| b != 0).unwrap_or(bytes.len());
    bytes[first_nonzero..].to_vec()
}

pub(super) fn to_big_endian_usize(value: usize) -> Vec<u8> {
    let bytes = value.to_be_bytes();
    let first_nonzero = bytes.iter().position(|&b| b != 0).unwrap_or(bytes.len());
    bytes[first_nonzero..].to_vec()
}
