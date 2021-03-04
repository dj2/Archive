use crate::Error;
use crate::Headers;
use crate::Method;
use crate::uri;
use crate::Version;

use std::convert::TryFrom;
use std::convert::TryInto;

/// An HTTP request.
#[derive(Debug, PartialEq)]
pub struct Request {
    /// The HTTP method for the request.
    pub method: Method,
    /// The HTTP version for the request.
    pub version: Version,
    /// The URI for the request. This is strictly the path
    /// component. The query params are split into params.
    pub uri: String,
    /// The HTTP headers
    pub headers: Headers,
    /// Any body data attached to the request.
    pub body: String,
}
impl TryFrom<String> for Request {
    type Error = Error;

    fn try_from(req: String) -> Result<Self, Error> {
        let mut headers = Headers::new();
        let mut body = String::from("");

        let mut lines = req.lines();
        let (method, uri, version) = parse_request_line(lines.next())?;

        let mut found_break: bool = false;
        while let Some(l) = lines.next() {
            // Headers end at a blank line
            if l.is_empty() {
                found_break = true;
                break;
            }

            let (k, v) = parse_header_line(l)?;
            headers.insert(&k, &v);
        }
        // Don't appear to have found the end of the headers.
        if !found_break {
            return Err(Error::PartialRequest);
        }

        for l in lines {
            if !body.is_empty() {
                body += "\n";
            }
            body += l;
        }

        // If a content-length header was provided make sure we
        // have that much data.
        if let Some(v) = headers.get(Headers::CONTENT_LENGTH) {
            let len = v.parse::<usize>()?;
            if len > body.len() {
                return Err(Error::PartialRequest);
            }
        } else if !body.is_empty() {
            return Err(Error::Parse(format!(
                "Parsed a body without a content-length: {}",
                body
            )));
        }

        Ok(Self {
            method,
            version,
            uri,
            headers,
            body,
        })
    }
}

fn parse_request_line(o: Option<&str>) -> Result<(Method, String, Version), Error> {
    if let Some(s) = o {
        let mut words = s.split_whitespace();
        let method = words
            .next()
            .ok_or_else(|| Error::Parse("Missing HTTP method".to_string()))?;
        let uri = words
            .next()
            .ok_or_else(|| Error::Parse("Missing HTTP URI".to_string()))?;
        let version = words
            .next()
            .ok_or_else(|| Error::Parse("Missing HTTP version".to_string()))?;

        let uri = uri::decode(uri)?;

        Ok((method.try_into()?, uri, version.try_into()?))
    } else {
        Err(Error::Parse("Missing HTTP request line".to_string()))
    }
}

fn parse_header_line(s: &str) -> Result<(String, String), Error> {
    let mut parts = s.split(':');
    let key = parts
        .next()
        .ok_or_else(|| Error::Parse(format!("Invalid header specified: {}", s)))?;
    if key.is_empty() {
        return Err(Error::Parse(format!("Invalid header key provided: {}", s)));
    }

    let mut val = String::from("");
    if let Some(v) = parts.next() {
        val = v.trim().to_string();
    }
    Ok((key.to_string(), val))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_http_request() {
        let s = String::from(
            "GET /test%20url HTTP/1.1
Host: localhost
User-Agent: Archive
Accept: */*

",
        );

        let mut headers = Headers::new();
        headers.insert(Headers::HOST, "localhost");
        headers.insert(Headers::USER_AGENT, "Archive");
        headers.insert(Headers::ACCEPT, "*/*");

        let req: Result<Request, Error> = s.try_into();
        assert!(req.is_ok(), "{:?}", req);

        let req = req.unwrap();
        assert_eq!(Method::Get, req.method);
        assert_eq!(Version::V1_1, req.version);
        assert_eq!("/test url", req.uri);
        assert_eq!(headers, req.headers);
    }

    #[test]
    fn read_http_request_with_body() {
        let s = String::from(
            "GET /test%20url HTTP/1.1
Host: localhost
User-Agent: Archive
Content-Length: 20
Accept: */*

This is the body 123",
        );

        let mut headers = Headers::new();
        headers.insert(Headers::HOST, "localhost");
        headers.insert(Headers::USER_AGENT, "Archive");
        headers.insert(Headers::CONTENT_LENGTH, "20");
        headers.insert(Headers::ACCEPT, "*/*");

        let req: Result<Request, Error> = s.try_into();
        assert!(req.is_ok(), "{:?}", req);

        let req = req.unwrap();
        assert_eq!(Method::Get, req.method);
        assert_eq!(Version::V1_1, req.version);
        assert_eq!("/test url", req.uri);
        assert_eq!(headers, req.headers);
        assert_eq!("This is the body 123", req.body);
    }

    #[test]
    fn read_http_body_preserves_newlines() {
        let s = String::from(
            "GET /test%20url HTTP/1.1
Host: localhost
User-Agent: Archive
Content-Length: 22
Accept: */*

This is the body

1234",
        );

        let req: Result<Request, Error> = s.try_into();
        assert!(req.is_ok(), "{:?}", req);

        let req = req.unwrap();
        assert_eq!("This is the body\n\n1234", req.body);
    }

    #[test]
    fn read_http_request_partial() {
        let s = String::from(
            "GET /test%20url HTTP/1.1
Host: localhost
User-Agent: Archive",
        );

        let req: Result<Request, Error> = s.try_into();
        assert_eq!(Err(Error::PartialRequest), req);
    }

    #[test]
    fn read_http_request_partial_body() {
        let s = String::from(
            "GET /test%20url HTTP/1.1
Host: localhost
User-Agent: Archive
Content-Length: 20

Start ...",
        );

        let req: Result<Request, Error> = s.try_into();
        assert_eq!(Err(Error::PartialRequest), req);
    }

    #[test]
    fn read_http_header_without_value() {
        let s = String::from(
            "GET /test%20url HTTP/1.1
Host:
User-Agent: Archive
Accept: */*

",
        );

        let mut headers = Headers::new();
        headers.insert(Headers::HOST, "");
        headers.insert(Headers::USER_AGENT, "Archive");
        headers.insert(Headers::ACCEPT, "*/*");

        let req: Result<Request, Error> = s.try_into();
        assert!(req.is_ok(), "{:?}", req);

        let req = req.unwrap();
        assert_eq!(headers, req.headers);
    }

    #[test]
    fn read_http_header_missing_key() {
        let s = String::from(
            "GET /test%20url HTTP/1.1
: localhost
User-Agent: Archive
Accept: */*

",
        );

        let req: Result<Request, Error> = s.try_into();
        assert!(req.is_err());
        assert_eq!(
            Err(Error::Parse(
                "Invalid header key provided: : localhost".to_string()
            )),
            req
        );
    }

    #[test]
    fn read_http_invalid_request_missing_version() {
        let s = String::from("GET /test%20url");
        let req: Result<Request, Error> = s.try_into();
        assert_eq!(Err(Error::Parse("Missing HTTP version".to_string())), req);
    }

    #[test]
    fn read_http_invalid_request_invalid_version() {
        let s = String::from("GET /test%20url HTTP/1.3");
        let req: Result<Request, Error> = s.try_into();
        assert_eq!(
            Err(Error::Parse("Unknown HTTP version: HTTP/1.3".to_string())),
            req
        );
    }

    #[test]
    fn read_http_invalid_request_missing_uri() {
        let s = String::from("GET ");
        let req: Result<Request, Error> = s.try_into();
        assert_eq!(Err(Error::Parse("Missing HTTP URI".to_string())), req);
    }

    #[test]
    fn read_http_invalid_request_missing_method() {
        let s = String::from("");
        let req: Result<Request, Error> = s.try_into();
        assert_eq!(
            Err(Error::Parse("Missing HTTP request line".to_string())),
            req
        );
    }

    #[test]
    fn read_http_invalid_request_invalid_method() {
        let s = String::from("INVALID /test%20url HTTP/1.2");
        let req: Result<Request, Error> = s.try_into();
        assert_eq!(
            Err(Error::Parse("Unknown HTTP method: INVALID".to_string())),
            req
        );
    }
}
