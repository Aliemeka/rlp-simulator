#[cfg(test)]
mod rlp_basic_types {
    use rlp_simulator::rlp::{Decodable, Encodable, Rlp, RlpStream};

    // ─── Helpers ───

    fn encode<T: Encodable>(val: &T) -> Vec<u8> {
        let mut stream = RlpStream::new();
        stream.append(val);
        stream.out()
    }

    fn decode<T: Decodable>(bytes: &[u8]) -> T {
        let rlp = Rlp::new(bytes);
        T::decode(&rlp).unwrap()
    }

    // ─── Empty string ───

    #[test]
    fn encode_empty_string() {
        // Empty string → [0x80]
        assert_eq!(encode(&""), vec![0x80]);
    }

    #[test]
    fn decode_empty_string() {
        assert_eq!(decode::<String>(&[0x80]), "");
    }

    // ─── Boolean false ───

    #[test]
    fn encode_bool_false() {
        // false → [0x80] (empty byte string)
        assert_eq!(encode(&false), vec![0x80]);
    }

    #[test]
    fn encode_bool_true() {
        // true → [0x01]
        assert_eq!(encode(&true), vec![0x01]);
    }

    #[test]
    fn decode_bool_false() {
        assert!(!decode::<bool>(&[0x80]));
    }

    #[test]
    fn decode_bool_true() {
        assert!(decode::<bool>(&[0x01]));
    }

    // ─── Empty list ───

    #[test]
    fn encode_empty_list() {
        // Empty list → [0xc0]
        let mut stream = RlpStream::new();
        stream.begin_list(0);
        let encoded = stream.out();
        assert_eq!(encoded, vec![0xc0]);
    }

    // ─── Short string (<= 55 bytes) ───

    #[test]
    fn encode_short_string() {
        // "hello" → [0x85, 'h', 'e', 'l', 'l', 'o']
        let mut expected = vec![0x80 + 5];
        expected.extend_from_slice(b"hello");
        assert_eq!(encode(&"hello"), expected);
    }

    #[test]
    fn decode_short_string() {
        let mut bytes = vec![0x80 + 5];
        bytes.extend_from_slice(b"hello");
        assert_eq!(decode::<String>(&bytes), "hello");
    }

    #[test]
    fn encode_decode_short_string_roundtrip() {
        let original = "the quick brown fox";
        let encoded = encode(&original);
        assert_eq!(decode::<String>(&encoded), original);
    }

    // ─── Long string (> 55 bytes) ───

    #[test]
    fn encode_long_string() {
        // 56-byte string: prefix is 0xb8 (0xb7 + 1 length byte), then 56, then data
        let s = "a".repeat(56);
        let encoded = encode(&s.as_str());
        assert_eq!(encoded[0], 0xb8); // 0xb7 + 1 byte to express length
        assert_eq!(encoded[1], 56); // length = 56
        assert_eq!(&encoded[2..], s.as_bytes());
    }

    #[test]
    fn decode_long_string() {
        let s = "a".repeat(56);
        let encoded = encode(&s.as_str());
        assert_eq!(decode::<String>(&encoded), s);
    }

    #[test]
    fn encode_decode_long_string_roundtrip() {
        let original = "x".repeat(200);
        let encoded = encode(&original.as_str());
        assert_eq!(decode::<String>(&encoded), original);
    }

    // ─── Single byte (0x00–0x7f) ───

    #[test]
    fn encode_single_byte_zero() {
        // 0 as u64 → [0x80] (zero integer = empty string in RLP)
        assert_eq!(encode(&0u64), vec![0x80]);
    }

    #[test]
    fn encode_single_byte_low() {
        // values 0x01–0x7f encode as their own single byte
        assert_eq!(encode(&0x42u64), vec![0x42]);
        assert_eq!(encode(&0x01u64), vec![0x01]);
        assert_eq!(encode(&0x7fu64), vec![0x7f]);
    }

    #[test]
    fn decode_single_byte_low() {
        assert_eq!(decode::<u64>(&[0x42]), 0x42);
        assert_eq!(decode::<u64>(&[0x01]), 0x01);
        assert_eq!(decode::<u64>(&[0x7f]), 0x7f);
    }

    #[test]
    fn encode_decode_single_byte_roundtrip() {
        for val in [0x00u64, 0x01, 0x42, 0x7f] {
            let encoded = encode(&val);
            assert_eq!(decode::<u64>(&encoded), val);
        }
    }
}
