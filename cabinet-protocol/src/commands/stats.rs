use crate::commands::KeyWord;
use elyze::bytes::primitives::whitespace::OptionalWhitespaces;
use elyze::errors::ParseResult;
use elyze::recognizer::recognize;
use elyze::scanner::Scanner;
use elyze::visitor::Visitor;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Stats;

impl Visitor<'_, u8> for Stats {
    fn accept(scanner: &mut Scanner<'_, u8>) -> ParseResult<Self> {
        recognize(KeyWord::Stats, scanner)?;
        OptionalWhitespaces::accept(scanner)?;
        Ok(Stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_stats() {
        let data = b"stats   ";
        let mut scanner = Scanner::new(data);
        let result = Stats::accept(&mut scanner);
        assert!(result.is_ok());
    }
}
