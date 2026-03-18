/// RLP encoding rules:
///
/// Single byte:    0x00..=0x7f → byte itself (no prefix)
/// Empty string:   [] → [0x80]
/// Short string:   1-55 bytes → [0x80 + len, ...data]
/// Long string:    56+ bytes → [0xb7 + len_of_len, ...len (big-endian), ...data]
/// Empty list:     [] → [0xc0]
/// Short list:     payload 1-55 bytes → [0xc0 + payload_len, ...items]
/// Long list:      payload 56+ bytes → [0xf7 + len_of_len, ...payload_len (big-endian), ...items]
use std::fmt;

mod encode;
mod decode;

#[derive(Debug)]
pub enum DecoderError {
    UnexpectedEnd,
    InvalidPrefix(u8),
    LeadingZeroInLength,
    NonCanonicalSingleByte,
    InvalidUtf8,
    InvalidInteger,
    ExpectedList,
    IndexOutOfBounds,
}

impl fmt::Display for DecoderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecoderError::UnexpectedEnd => write!(f, "unexpected end of input"),
            DecoderError::InvalidPrefix(b) => write!(f, "invalid prefix byte: 0x{:02x}", b),
            DecoderError::LeadingZeroInLength => write!(f, "leading zero in length"),
            DecoderError::NonCanonicalSingleByte => {
                write!(f, "non-canonical encoding for single byte")
            }
            DecoderError::InvalidUtf8 => write!(f, "invalid UTF-8"),
            DecoderError::InvalidInteger => write!(f, "invalid integer encoding"),
            DecoderError::ExpectedList => write!(f, "expected an RLP list"),
            DecoderError::IndexOutOfBounds => write!(f, "list index out of bounds"),
        }
    }
}

impl std::error::Error for DecoderError {}

// ─── Traits ───

/// Types that can be RLP encoded.
pub trait Encodable {
    fn rlp_append(&self, s: &mut RlpStream);
}

/// Types that can be RLP decoded.
pub trait Decodable: Sized {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError>;
}

// ─── Encoder ───

pub struct RlpStream {
    buffer: Vec<u8>,
    /// Tracks nested lists: each entry is the start index of that list's payload in `buffer`.
    list_stack: Vec<usize>,
}

impl RlpStream {
    pub fn new() -> Self {
        RlpStream {
            buffer: Vec::new(),
            list_stack: Vec::new(),
        }
    }

    /// Start a list with a known number of items.
    pub fn begin_list(&mut self, _len: usize) -> &mut Self {
        self.list_stack.push(self.buffer.len());
        self
    }

    /// Finish the current list by wrapping its payload with the correct header.
    fn finish_list(&mut self) {
        if let Some(start) = self.list_stack.pop() {
            let payload: Vec<u8> = self.buffer.drain(start..).collect();
            let header = encode::encode_list_header(payload.len());
            self.buffer.extend_from_slice(&header);
            self.buffer.extend_from_slice(&payload);
        }
    }

    /// Append a value that implements `Encodable`.
    pub fn append<T: Encodable>(&mut self, value: &T) -> &mut Self {
        value.rlp_append(self);
        self
    }

    /// Append raw already-encoded bytes directly to the stream.
    pub fn append_raw(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
    }

    /// Finalize and return the encoded bytes.
    pub fn out(mut self) -> Vec<u8> {
        while !self.list_stack.is_empty() {
            self.finish_list();
        }
        self.buffer
    }
}

// ─── Decoder ───

pub struct Rlp<'a> {
    data: &'a [u8],
}

impl<'a> Rlp<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Rlp { data }
    }

    /// Decode this RLP item as a value implementing `Decodable`.
    pub fn as_val<T: Decodable>(&self) -> Result<T, DecoderError> {
        T::decode(self)
    }

    /// Decode the item at the given index within this list.
    pub fn val_at<T: Decodable>(&self, index: usize) -> Result<T, DecoderError> {
        let item = self.at(index)?;
        T::decode(&item)
    }

    /// Get the RLP item at the given index within this list.
    pub fn at(&self, index: usize) -> Result<Rlp<'a>, DecoderError> {
        let (payload_offset, payload_len) = self.list_payload()?;
        let payload = &self.data[payload_offset..payload_offset + payload_len];

        let mut offset = 0;
        let mut current = 0;

        while offset < payload.len() {
            let item_len = decode::item_total_len(&payload[offset..])?;

            if current == index {
                return Ok(Rlp::new(&payload[offset..offset + item_len]));
            }

            offset += item_len;
            current += 1;
        }

        Err(DecoderError::IndexOutOfBounds)
    }

    /// Return the raw bytes of this item (excluding RLP header).
    pub fn data(&self) -> Result<&'a [u8], DecoderError> {
        let (offset, len) = decode::string_payload(self.data)?;
        Ok(&self.data[offset..offset + len])
    }

    /// Extract the payload offset and length if this is a list.
    fn list_payload(&self) -> Result<(usize, usize), DecoderError> {
        if self.data.is_empty() {
            return Err(DecoderError::UnexpectedEnd);
        }

        let prefix = self.data[0];

        match prefix {
            0xc0..=0xf7 => {
                let len = (prefix - 0xc0) as usize;
                Ok((1, len))
            }
            0xf8..=0xff => {
                let len_of_len = (prefix - 0xf7) as usize;
                if self.data.len() < 1 + len_of_len {
                    return Err(DecoderError::UnexpectedEnd);
                }
                let len_bytes = &self.data[1..1 + len_of_len];
                if len_bytes[0] == 0 {
                    return Err(DecoderError::LeadingZeroInLength);
                }
                let len = decode::from_big_endian(len_bytes);
                Ok((1 + len_of_len, len))
            }
            _ => Err(DecoderError::ExpectedList),
        }
    }
}

impl fmt::Display for Rlp<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.data {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}
