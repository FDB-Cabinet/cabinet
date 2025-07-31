use crate::commands::{Data, KeyWord};
use elyze::bytes::primitives::whitespace::{OptionalWhitespaces, Whitespaces};
use elyze::errors::ParseResult;
use elyze::recognizer::recognize;
use elyze::scanner::Scanner;
use elyze::visitor::Visitor;
use std::fmt::{Debug, Formatter};

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Put<'a> {
    pub key: &'a [u8],
    pub value: &'a [u8],
}

impl Debug for Put<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Put")
            .field("key", &String::from_utf8_lossy(self.key))
            .field("value", &String::from_utf8_lossy(self.key))
            .finish()
    }
}

impl<'a> Visitor<'a, u8> for Put<'a> {
    fn accept(scanner: &mut Scanner<'a, u8>) -> ParseResult<Self> {
        recognize(KeyWord::Put, scanner)?;
        Whitespaces::accept(scanner)?;
        let key = Data::accept(scanner)?.data;
        Whitespaces::accept(scanner)?;
        let value = Data::accept(scanner)?.data;
        OptionalWhitespaces::accept(scanner)?;
        Ok(Put { key, value })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_put() {
        let mut scanner = Scanner::new(br#"put "key" "value""#);
        let put = Put::accept(&mut scanner).expect("Unable to parse put command");
        assert_eq!(put.key, b"key");
        assert_eq!(put.value, b"value");
    }
}
