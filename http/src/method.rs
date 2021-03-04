use crate::Error;
use std::convert::TryFrom;
use std::fmt;

/// A HTTP method.
#[derive(Debug, PartialEq)]
pub enum Method {
    /// The GET method.
    ///
    /// Requests a representation of a specified resource.
    Get,
    /// The HEAD method.
    ///
    /// Response identical to GET but without the body.
    Head,
    /// The POST method.
    ///
    /// Used to submit an entity to the specified resource.
    Post,
    /// The PUT method.
    ///
    /// Replaces all current representations of the resource with the payload.
    Put,
    /// The DELETE method.
    ///
    /// Deletes the specified resource.
    Delete,
    /// The CONNECT method.
    ///
    /// Establishes a tunnel to the server identified by the resource.
    Connect,
    /// The OPTIONS method.
    ///
    /// Describes the communication options for the resource.
    Options,
    /// The TRACE method.
    ///
    /// Performs a message loop-back test along the path to the resource.
    Trace,
    /// The PATCH method.
    ///
    /// Applies a partial modification to the resource.
    Patch,
}
impl TryFrom<&str> for Method {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self, Error> {
        match s {
            "GET" => Ok(Self::Get),
            "HEAD" => Ok(Self::Head),
            "POST" => Ok(Self::Post),
            "PUT" => Ok(Self::Put),
            "DELETE" => Ok(Self::Delete),
            "CONNECT" => Ok(Self::Connect),
            "OPTIONS" => Ok(Self::Options),
            "TRACE" => Ok(Self::Trace),
            "PATCH" => Ok(Self::Patch),
            _ => Err(Error::Parse(format!("Unknown HTTP method: {}", s))),
        }
    }
}
impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Get => write!(f, "GET"),
            Self::Head => write!(f, "HEAD"),
            Self::Post => write!(f, "POST"),
            Self::Put => write!(f, "PUT"),
            Self::Delete => write!(f, "DELETE"),
            Self::Connect => write!(f, "CONNECT"),
            Self::Options => write!(f, "OPTIONS"),
            Self::Trace => write!(f, "TRACE"),
            Self::Patch => write!(f, "PATCH"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryInto;

    type M = Result<Method, Error>;

    #[test]
    fn parse_method_get() {
        let m: M = "GET".try_into();
        assert_eq!(m, Ok(Method::Get))
    }
    #[test]
    fn parse_method_head() {
        let m: M = "HEAD".try_into();
        assert_eq!(m, Ok(Method::Head))
    }
    #[test]
    fn parse_method_post() {
        let m: M = "POST".try_into();
        assert_eq!(m, Ok(Method::Post))
    }
    #[test]
    fn parse_method_put() {
        let m: M = "PUT".try_into();
        assert_eq!(m, Ok(Method::Put))
    }
    #[test]
    fn parse_method_delete() {
        let m: M = "DELETE".try_into();
        assert_eq!(m, Ok(Method::Delete))
    }
    #[test]
    fn parse_method_connect() {
        let m: M = "CONNECT".try_into();
        assert_eq!(m, Ok(Method::Connect))
    }
    #[test]
    fn parse_method_options() {
        let m: M = "OPTIONS".try_into();
        assert_eq!(m, Ok(Method::Options))
    }
    #[test]
    fn parse_method_trace() {
        let m: M = "TRACE".try_into();
        assert_eq!(m, Ok(Method::Trace))
    }
    #[test]
    fn parse_method_patch() {
        let m: M = "PATCH".try_into();
        assert_eq!(m, Ok(Method::Patch))
    }
    #[test]
    fn parse_method_invalid() {
        let m: M = "INVALID".try_into();
        assert_eq!(
            m,
            Err(Error::Parse("Unknown HTTP method: INVALID".to_string()))
        )
    }

    #[test]
    fn to_string() {
        assert_eq!("GET", Method::Get.to_string());
    }
}
