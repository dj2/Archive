use crate::error::Error;
use std::convert::TryFrom;

/// The HTTP version.
#[derive(Debug, PartialEq)]
pub enum Version {
    /// Version 1.0
    V1_0,
    /// Version 1.1
    V1_1,
    /// Version 2.0
    V2_0,
}
impl TryFrom<&str> for Version {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self, Error> {
        match s {
            "HTTP/1.0" => Ok(Self::V1_0),
            "HTTP/1.1" => Ok(Self::V1_1),
            "HTTP/2.0" => Ok(Self::V2_0),
            _ => Err(Error::Parse(format!("Unknown HTTP version: {}", s))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryInto;

    type V = Result<Version, Error>;

    #[test]
    fn parse_version_1_0() {
        let v: V = "HTTP/1.0".try_into();
        assert_eq!(v, Ok(Version::V1_0))
    }
    #[test]
    fn parse_version_1_1() {
        let v: V = "HTTP/1.1".try_into();
        assert_eq!(v, Ok(Version::V1_1))
    }
    #[test]
    fn parse_version_2_0() {
        let v: V = "HTTP/2.0".try_into();
        assert_eq!(v, Ok(Version::V2_0))
    }
    #[test]
    fn parse_version_invalid() {
        let v: V = "HTTP/1.3".try_into();
        assert_eq!(
            v,
            Err(Error::Parse("Unknown HTTP version: HTTP/1.3".to_string()))
        )
    }
}
