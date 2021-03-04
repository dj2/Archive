use std::fmt;

/// HTTP Status Codes
#[derive(Clone, Debug, PartialEq)]
pub enum Status {
  Continue,
  SwitchingProtocol,
  Ok,
  Created,
  Accepted,
  NonAuthoritative,
  NoContent,
  ResetContent,
  PartialContent,
  MultipleChoices,
  MovedPermanently,
  Found,
  SeeOther,
  NotModified,
  UseProxy,
  TemporaryRedirect,
  BadRequest,
  Unauthorized,
  PaymentRequired,
  Forbidden,
  NotFound,
  MethodNotAllowed,
  NotAcceptable,
  ProxyAuthenticationRequired,
  RequestTimeout,
  Conflict,
  Gone,
  LengthRequired,
  PreconditionFailed,
  PayloadTooLarge,
  UriTooLong,
  UnsupportedMediaType,
  RangeNotSatisified,
  ExpectationFailed,
  UpgradeRequired,
  InternalServerError,
  NotImplemented,
  BadGateway,
  ServiceUnavailable,
  GatewayTimeout,
  HttpVersionNotSupported,
}
impl fmt::Display for Status {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match *self {
      Self::Continue => write!(f, "100 Continue"),
      Self::SwitchingProtocol => write!(f, "101 Switching Protocol"),
      Self::Ok => write!(f, "200 OK"),
      Self::Created => write!(f, "201 Created"),
      Self::Accepted => write!(f, "202 Accepted"),
      Self::NonAuthoritative => write!(f, "203 Non-Authoritative Information"),
      Self::NoContent => write!(f, "204 No Content"),
      Self::ResetContent => write!(f, "205 Reset Content"),
      Self::PartialContent => write!(f, "206 Partial Content"),
      Self::MultipleChoices => write!(f, "300 Multiple Choices"),
      Self::MovedPermanently => write!(f, "301 Moved Permanently"),
      Self::Found => write!(f, "302 Found"),
      Self::SeeOther => write!(f, "303 See Other"),
      Self::NotModified => write!(f, "304 Not Modified"),
      Self::UseProxy => write!(f, "305 Use Proxy"),
      Self::TemporaryRedirect => write!(f, "307 Temporary Redirect"),
      Self::BadRequest => write!(f, "400 Bad Request"),
      Self::Unauthorized => write!(f, "401 Unauthorized"),
      Self::PaymentRequired => write!(f, "402 Payment Required"),
      Self::Forbidden => write!(f, "403 Forbidden"),
      Self::NotFound => write!(f, "404 Not Found"),
      Self::MethodNotAllowed => write!(f, "405 Method Not Allowed"),
      Self::NotAcceptable => write!(f, "406 Not Acceptable"),
      Self::ProxyAuthenticationRequired => write!(f, "407 Proxy Authentication Required"),
      Self::RequestTimeout => write!(f, "408 Request Timeout"),
      Self::Conflict => write!(f, "409 Conflict"),
      Self::Gone => write!(f, "410 Gone"),
      Self::LengthRequired => write!(f, "411 Length Required"),
      Self::PreconditionFailed => write!(f, "412 Precondition Failed"),
      Self::PayloadTooLarge => write!(f, "413 Payload Too Large"),
      Self::UriTooLong => write!(f, "414 URI Too Long"),
      Self::UnsupportedMediaType => write!(f, "415 Unsupported Media Type"),
      Self::RangeNotSatisified => write!(f, "416 Range Not Satisfiable"),
      Self::ExpectationFailed => write!(f, "417 Expectation Failed"),
      Self::UpgradeRequired => write!(f, "426 Upgrade Required"),
      Self::InternalServerError => write!(f, "500 Internal Server Error"),
      Self::NotImplemented => write!(f, "501 Not Implemented"),
      Self::BadGateway => write!(f, "502 Bad Gateway"),
      Self::ServiceUnavailable => write!(f, "503 Service Unavailable"),
      Self::GatewayTimeout => write!(f, "504 Gateway Timeout"),
      Self::HttpVersionNotSupported => write!(f, "505 HTTP Version Not Supported"),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn to_string() {
    assert_eq!("204 No Content", Status::NoContent.to_string());
  }
}
