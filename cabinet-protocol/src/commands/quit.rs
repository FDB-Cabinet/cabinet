use crate::commands::KeyWord;
use elyze::bytes::primitives::whitespace::OptionalWhitespaces;
use elyze::errors::ParseResult;
use elyze::recognizer::recognize;
use elyze::scanner::Scanner;
use elyze::visitor::Visitor;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Quit;

impl Visitor<'_, u8> for Quit {
    fn accept(scanner: &mut Scanner<'_, u8>) -> ParseResult<Self> {
        recognize(KeyWord::Quit, scanner)?;
        OptionalWhitespaces::accept(scanner)?;
        Ok(Quit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_clear() {
        let data = b"quit   ";
        let mut scanner = Scanner::new(data);
        let result = Quit::accept(&mut scanner);
        assert!(result.is_ok());
    }
}
