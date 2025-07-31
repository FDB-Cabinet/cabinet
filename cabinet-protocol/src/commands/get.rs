use crate::commands::{Data, KeyWord};
use elyze::bytes::primitives::whitespace::{OptionalWhitespaces, Whitespaces};
use elyze::errors::ParseResult;
use elyze::recognizer::recognize;
use elyze::scanner::Scanner;
use elyze::visitor::Visitor;
use std::fmt::Debug;

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Get<'a> {
    pub key: &'a [u8],
}

impl Debug for Get<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Get")
            .field("key", &String::from_utf8_lossy(self.key))
            .finish()
    }
}

// get "key"

impl<'a> Visitor<'a, u8> for Get<'a> {
    fn accept(scanner: &mut Scanner<'a, u8>) -> ParseResult<Self> {
        recognize(KeyWord::Get, scanner)?;
        Whitespaces::accept(scanner)?;
        let key = Data::accept(scanner)?.data;
        OptionalWhitespaces::accept(scanner)?;
        Ok(Self { key })
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::get::Get;
    use elyze::scanner::Scanner;
    use elyze::visitor::Visitor;

    #[test]
    fn parse_get_command() {
        let data = br#"get      "key"    "#;
        let mut scanner = Scanner::new(data);
        let result = Get::accept(&mut scanner);
        dbg!(&result);
        assert!(matches!(result, Ok(Get { key: b"key" })))
    }
}
