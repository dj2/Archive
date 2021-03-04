use crate::error::Error;

const fn from_hex(c: u8) -> Option<u8> {
    match c {
        b'0'..=b'9' => Some(c - b'0'),
        b'a'..=b'f' => Some(c - b'a' + 10),
        b'A'..=b'F' => Some(c - b'A' + 10),
        _ => None,
    }
}

/// Decodes URIs as per <https://url.spec.whatwg.org/>
/// # Errors
/// Returns Error if the URI is not valid UTF8
pub fn decode(s: &str) -> Result<String, Error> {
    let mut out: Vec<u8> = Vec::with_capacity(s.len());
    let mut bytes = s.as_bytes().iter();
    while let Some(c) = bytes.next() {
        match c {
            b'%' => {
                if let Some(a_char) = bytes.next() {
                    if let Some(a_val) = from_hex(*a_char) {
                        if let Some(b_char) = bytes.next() {
                            if let Some(b_val) = from_hex(*b_char) {
                                out.push((a_val << 4) | b_val);
                            } else {
                                out.push(*c);
                                out.push(*a_char);
                                out.push(*b_char);
                            }
                        } else {
                            out.push(*c);
                            out.push(*a_char);
                        }
                    } else {
                        out.push(*c);
                        out.push(*a_char);
                    }
                } else {
                    out.push(*c);
                }
            }
            _ => out.push(*c),
        }
    }
    String::from_utf8(out).map_err(|e| Error::Parse(format!("{:?}", e)))
}

#[cfg(test)]
mod test {
    use super::decode;

    #[test]
    fn it_decodes() {
        assert_eq!(
            Ok("MyUrl #%OtherChars".to_string()),
            decode("MyUrl%20%23%25OtherChars")
        );
    }

    #[test]
    fn it_decodes_lone_percent() {
        assert_eq!(Ok("MyUrl%tt".to_string()), decode("MyUrl%tt"));
    }

    #[test]
    fn it_decodes_percent_plus_char() {
        assert_eq!(Ok("MyUrl%az".to_string()), decode("MyUrl%az"));
    }

    #[test]
    fn it_decodes_percent_at_end() {
        assert_eq!(Ok("MyUrl%".to_string()), decode("MyUrl%"));
    }

    #[test]
    fn it_decodes_percent_and_char_at_end() {
        assert_eq!(Ok("MyUrl%a".to_string()), decode("MyUrl%a"));
    }

    #[test]
    fn it_decodes_upper_and_lower_hex() {
        assert_eq!(
            Ok("My Url LOOK".to_string()),
            decode("%4d%79%20%55%72%6c%20%4C%4F%4F%4B")
        );
    }
}
