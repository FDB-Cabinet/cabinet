use crate::commands::KeyWord;
use elyze::bytes::primitives::whitespace::OptionalWhitespaces;
use elyze::errors::ParseResult;
use elyze::recognizer::recognize;
use elyze::scanner::Scanner;
use elyze::visitor::Visitor;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Clear;

impl Visitor<'_, u8> for Clear {
    fn accept(scanner: &mut Scanner<'_, u8>) -> ParseResult<Self> {
        recognize(KeyWord::Clear, scanner)?;
        OptionalWhitespaces::accept(scanner)?;
        Ok(Clear)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_clear() {
        let data = b"clear   ";
        let mut scanner = Scanner::new(data);
        let result = Clear::accept(&mut scanner);
        assert!(result.is_ok());
    }
}
