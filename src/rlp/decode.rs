use super::{Decodable, DecoderError, Rlp};

impl Decodable for bool {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        let bytes = rlp.data()?;
        match bytes {
            [] => Ok(false),
            [0x01] => Ok(true),
            _ => Err(DecoderError::InvalidPrefix(bytes[0])),
        }
    }
}

impl Decodable for u64 {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        let bytes = rlp.data()?;

        if bytes.is_empty() {
            return Ok(0);
        }

        if bytes.len() > 1 && bytes[0] == 0 {
            return Err(DecoderError::InvalidInteger);
        }

        if bytes.len() > 8 {
            return Err(DecoderError::InvalidInteger);
        }

        Ok(from_big_endian(bytes) as u64)
    }
}

impl Decodable for Vec<u8> {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        Ok(rlp.data()?.to_vec())
    }
}

impl Decodable for String {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        let bytes = rlp.data()?;
        String::from_utf8(bytes.to_vec()).map_err(|_| DecoderError::InvalidUtf8)
    }
}

/// Extract payload offset and length for a string item.
pub(super) fn string_payload(data: &[u8]) -> Result<(usize, usize), DecoderError> {
    if data.is_empty() {
        return Err(DecoderError::UnexpectedEnd);
    }

    let prefix = data[0];

    match prefix {
        0x00..=0x7f => Ok((0, 1)),

        0x80..=0xb7 => {
            let len = (prefix - 0x80) as usize;
            Ok((1, len))
        }

        0xb8..=0xbf => {
            let len_of_len = (prefix - 0xb7) as usize;
            if data.len() < 1 + len_of_len {
                return Err(DecoderError::UnexpectedEnd);
            }
            let len_bytes = &data[1..1 + len_of_len];
            if len_bytes[0] == 0 {
                return Err(DecoderError::LeadingZeroInLength);
            }
            let len = from_big_endian(len_bytes);
            Ok((1 + len_of_len, len))
        }

        _ => Err(DecoderError::InvalidPrefix(prefix)),
    }
}

/// Calculate the total byte length of an RLP item (header + payload).
pub(super) fn item_total_len(data: &[u8]) -> Result<usize, DecoderError> {
    if data.is_empty() {
        return Err(DecoderError::UnexpectedEnd);
    }

    let prefix = data[0];

    match prefix {
        0x00..=0x7f => Ok(1),

        0x80..=0xb7 => {
            let len = (prefix - 0x80) as usize;
            Ok(1 + len)
        }

        0xb8..=0xbf => {
            let len_of_len = (prefix - 0xb7) as usize;
            if data.len() < 1 + len_of_len {
                return Err(DecoderError::UnexpectedEnd);
            }
            let len = from_big_endian(&data[1..1 + len_of_len]);
            Ok(1 + len_of_len + len)
        }

        0xc0..=0xf7 => {
            let len = (prefix - 0xc0) as usize;
            Ok(1 + len)
        }

        0xf8..=0xff => {
            let len_of_len = (prefix - 0xf7) as usize;
            if data.len() < 1 + len_of_len {
                return Err(DecoderError::UnexpectedEnd);
            }
            let len = from_big_endian(&data[1..1 + len_of_len]);
            Ok(1 + len_of_len + len)
        }
    }
}

pub(super) fn from_big_endian(bytes: &[u8]) -> usize {
    let mut result: usize = 0;
    for &b in bytes {
        result = (result << 8) | b as usize;
    }
    result
}
