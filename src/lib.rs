pub mod rlp;
use rlp::{Decodable, Encodable, Rlp, RlpStream};

// Simulates an ethereum transaction object
#[derive(Debug)]
pub struct Transaction {
    pub nonce: u64,
    pub to: Vec<u8>,
    pub value: u64,
}

impl Encodable for Transaction {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(3);
        s.append(&self.nonce);
        s.append(&self.to);
        s.append(&self.value);
    }
}

impl Decodable for Transaction {
    fn decode(rlp: &Rlp) -> Result<Self, rlp::DecoderError> {
        Ok(Transaction {
            nonce: rlp.val_at(0)?,
            to: rlp.val_at(1)?,
            value: rlp.val_at(2)?,
        })
    }
}

pub fn encode_string(s: String) -> RlpStream {
    let mut stream = RlpStream::new();
    stream.append(&s);
    stream
}

pub fn decode_string(rlp: &Rlp) -> Result<String, rlp::DecoderError> {
    rlp.as_val()
}
