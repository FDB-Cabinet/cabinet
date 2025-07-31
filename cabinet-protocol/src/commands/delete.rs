use crate::commands::{Data, KeyWord};
use elyze::bytes::primitives::whitespace::{OptionalWhitespaces, Whitespaces};
use elyze::errors::ParseResult;
use elyze::recognizer::recognize;
use elyze::scanner::Scanner;
use elyze::visitor::Visitor;
use std::fmt::Debug;

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct Delete<'a> {
    pub key: &'a [u8],
}

impl Debug for Delete<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Delete")
            .field("key", &String::from_utf8_lossy(self.key))
            .finish()
    }
}

impl<'a> Visitor<'a, u8> for Delete<'a> {
    fn accept(scanner: &mut Scanner<'a, u8>) -> ParseResult<Self> {
        recognize(KeyWord::Delete, scanner)?;
        Whitespaces::accept(scanner)?;
        let key = Data::accept(scanner)?.data;
        OptionalWhitespaces::accept(scanner)?;
        Ok(Delete { key })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use elyze::scanner::Scanner;
    use elyze::visitor::Visitor;
    #[test]
    fn test_delete() {
        let mut scanner = Scanner::new(br#"DELETE "key""#);
        let delete = Delete::accept(&mut scanner).expect("Unable to parse DELETE command");
        assert_eq!(delete.key, b"key");
    }
}
