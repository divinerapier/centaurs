use std::collections::HashMap;

const CHARSET: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

lazy_static::lazy_static! {
    static ref INDEX: HashMap<u8, u8> = {
        CHARSET.iter().enumerate().map(|(index, &value)|{(value, index as u8)}).collect::<HashMap<u8, u8>>()
    };


    static ref VALUE: Vec<u8> = {
        vec!{
            b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9',
            b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i', b'j',
            b'k', b'l', b'm', b'n', b'o', b'p', b'q', b'r', b's', b't',
            b'u', b'v', b'w', b'x', b'y', b'z', b'A', b'B', b'C', b'D',
            b'E', b'F', b'G', b'H', b'I', b'J', b'K', b'L', b'M', b'N',
            b'O', b'P', b'Q', b'R', b'S', b'T', b'U', b'V', b'W', b'X',
            b'Y', b'Z',
        }
    };
}

pub fn value2index(v: u8) -> Option<u8> {
    Some(*INDEX.get(&v)?)
}

pub fn index2value(v: u8) -> Option<u8> {
    Some(*VALUE.get(v as usize)?)
}

pub fn contains(v: u8) -> bool {
    value2index(v).is_some()
}

pub fn decode<S>(value: S) -> Option<u64>
where
    S: AsRef<str>,
{
    decode_bytes(value.as_ref().as_bytes())
}

fn decode_bytes(bytes: &[u8]) -> Option<u64> {
    bytes.iter().try_fold(0u64, |result, c| {
        INDEX.get(c).and_then(|c| {
            result
                .checked_mul(CHARSET.len() as u64)
                .and_then(|v| v.checked_add(*c as u64))
        })
    })
}

pub fn encode(mut value: u64) -> String {
    if (value as usize) < CHARSET.len() {
        let r = VALUE[(value as usize)];
        let s = unsafe { String::from_utf8_unchecked(vec![r]) };
        return s;
    }
    let mut buf = [0u8; 16];
    let mut start_at = buf.len() - 1;
    while value > 0 {
        let index = value % CHARSET.len() as u64;
        value /= CHARSET.len() as u64;
        buf[start_at] = VALUE[(index as usize)];
        start_at -= 1;
    }
    unsafe { String::from_utf8_unchecked(buf[start_at + 1..].to_vec()) }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_decode() {
        fn check(_name: &str, value: &str, expected: impl Into<Option<u64>>) {
            let result = decode(value);
            assert_eq!(expected.into(), result);
        }

        check("Decode-0", "0", 0);
        check("Decode-1", "1", 1);
        check("Decode-9", "9", 9);
        check("Decode-a", "a", 10);
        check("Decode-z", "z", 35);
        check("Decode-A", "A", 36);
        check("Decode-Z", "Z", 61);
        check("Decode-10", "10", 62);
        check("Decode-lYGhA16ahyf", "lYGhA16ahyf", 18446744073709551615);
        check("Decode-lYGhA16ahyg", "lYGhA16ahyg", None);
    }

    #[test]
    fn test_encode() {
        fn check(_name: &str, value: u64, expected: impl Into<String>) {
            let result = encode(value);
            assert_eq!(expected.into(), result);
        }

        check("Encode-0", 0, "0");
        check("Encode-1", 1, "1");
        check("Encode-9", 9, "9");
        check("Encode-10", 10, "a");
        check("Decode-35", 35, "z");
        check("Decode-36", 36, "A");
        check("Decode-61", 61, "Z");
        check("Decode-62", 62, "10");
        check("Decode-lYGhA16ahyf", 18446744073709551615, "lYGhA16ahyf");
    }

    #[test]
    fn test_contains() {
        for &c in CHARSET {
            assert!(contains(c));
        }

        assert!(!contains(b' '));
        assert!(!contains(b'-'));
        assert!(!contains(b'_'));
    }
}
