use crate::commands::{Data, KeyWord};
use elyze::bytes::primitives::whitespace::{OptionalWhitespaces, Whitespaces};
use elyze::errors::ParseResult;
use elyze::recognizer::recognize;
use elyze::scanner::Scanner;
use elyze::visitor::Visitor;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Auth<'a> {
    pub tenant: &'a str,
}

impl<'a> Visitor<'a, u8> for Auth<'a> {
    fn accept(scanner: &mut Scanner<'a, u8>) -> ParseResult<Self> {
        recognize(KeyWord::Auth, scanner)?;
        Whitespaces::accept(scanner)?;
        let tenant_bytes = Data::accept(scanner)?.data;
        let tenant = std::str::from_utf8(tenant_bytes)?;
        OptionalWhitespaces::accept(scanner)?;
        Ok(Auth { tenant })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_auth() {
        let mut scanner = Scanner::new(br#"auth "test""#);
        let auth = Auth::accept(&mut scanner).unwrap();
        assert_eq!(auth.tenant, "test");
    }
}
