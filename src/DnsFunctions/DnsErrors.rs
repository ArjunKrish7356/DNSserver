use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum DnsResolverError {
    NetworkError(std::io::Error),
    ParseError(String),
    NoQuestionFound,
    NoNameserverFound,
    ResolutionFailed,
}

impl fmt::Display for DnsResolverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DnsResolverError::NetworkError(err) => 
                write!(f, "Network error: {}", err),
            DnsResolverError::ParseError(details) => 
                write!(f, "Parsing error: {}", details),
            DnsResolverError::NoQuestionFound => 
                write!(f, "No DNS question found in the packet"),
            DnsResolverError::NoNameserverFound => 
                write!(f, "Unable to find a valid nameserver"),
            DnsResolverError::ResolutionFailed => 
                write!(f, "DNS resolution failed"),
        }
    }
}

impl Error for DnsResolverError {}

impl From<std::io::Error> for DnsResolverError {
    fn from(err: std::io::Error) -> Self {
        DnsResolverError::NetworkError(err)
    }
}