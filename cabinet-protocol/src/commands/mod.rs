use crate::commands::auth::Auth;
use crate::commands::clear::Clear;
use crate::commands::delete::Delete;
use crate::commands::get::Get;
use crate::commands::put::Put;
use crate::commands::stats::Stats;
use elyze::acceptor::Acceptor;
use elyze::bytes::components::groups::GroupKind;
use elyze::bytes::matchers::match_pattern;
use elyze::bytes::token::Token;
use elyze::errors::ParseError::UnexpectedToken;
use elyze::errors::ParseResult;
use elyze::matcher::Match;
use elyze::peek::{peek, UntilEnd};
use elyze::peeker::Peeker;
use elyze::recognizer::recognize;
use elyze::scanner::Scanner;
use elyze::visitor::Visitor;

pub mod auth;
pub mod clear;
pub mod delete;
pub mod get;
pub mod put;
pub mod quit;
pub mod stats;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Unknown;

impl<'a> Visitor<'a, u8> for Unknown {
    fn accept(scanner: &mut Scanner<'a, u8>) -> ParseResult<Self> {
        let peeker = Peeker::new(&scanner)
            .add_peekable(Token::Whitespace)
            .add_peekable(UntilEnd::default());
        let raw = peeker.peek()?;
        if let Some(peeked) = raw {
            scanner.bump_by(peeked.end_slice);
        }
        Ok(Unknown)
    }
}

// "fdsgfg"
struct Data<'a> {
    data: &'a [u8],
}

impl<'a> Visitor<'a, u8> for Data<'a> {
    fn accept(scanner: &mut Scanner<'a, u8>) -> ParseResult<Self> {
        let raw = peek(GroupKind::DoubleQuotes, scanner)?.ok_or(UnexpectedToken)?;
        scanner.bump_by(raw.end_slice);
        Ok(Data {
            data: raw.peeked_slice(),
        })
    }
}

pub enum KeyWord {
    Auth,
    Put,
    Get,
    Delete,
    Clear,
    Stats,
    Quit,
}

impl Match<u8> for KeyWord {
    fn is_matching(&self, data: &[u8]) -> (bool, usize) {
        match self {
            KeyWord::Auth => match_pattern(b"auth", data),
            KeyWord::Put => match_pattern(b"put", data),
            KeyWord::Get => match_pattern(b"get", data),
            KeyWord::Delete => match_pattern(b"delete", data),
            KeyWord::Clear => match_pattern(b"clear", data),
            KeyWord::Stats => match_pattern(b"stats", data),
            KeyWord::Quit => match_pattern(b"quit", data),
        }
    }

    fn size(&self) -> usize {
        match self {
            KeyWord::Auth => 4,
            KeyWord::Put => 3,
            KeyWord::Get => 3,
            KeyWord::Delete => 6,
            KeyWord::Clear => 5,
            KeyWord::Stats => 5,
            KeyWord::Quit => 4,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Command<'a> {
    Auth(Auth<'a>),
    Put(Put<'a>),
    Get(Get<'a>),
    Delete(Delete<'a>),
    Clear(Clear),
    Stats(Stats),
    Quit(quit::Quit),
    Unknown(Unknown),
}

impl<'a> Visitor<'a, u8> for Command<'a> {
    fn accept(scanner: &mut Scanner<'a, u8>) -> ParseResult<Self> {
        let accepted = Acceptor::new(scanner)
            .try_or(Command::Auth)?
            .try_or(Command::Get)?
            .try_or(Command::Put)?
            .try_or(Command::Delete)?
            .try_or(Command::Clear)?
            .try_or(Command::Stats)?
            .try_or(Command::Quit)?
            .try_or(Command::Unknown)?
            .finish()
            .ok_or(UnexpectedToken)?;
        let _ = recognize(Token::Ln, scanner);
        Ok(accepted)
    }
}

pub struct Commands<'a> {
    scanner: Scanner<'a, u8>,
}

impl<'a> Commands<'a> {
    pub fn new(commands: &'a [u8]) -> Self {
        Self {
            scanner: Scanner::new(commands),
        }
    }
}

impl<'a> Iterator for Commands<'a> {
    type Item = ParseResult<Command<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.scanner.is_empty() {
            return None;
        }

        let command = Command::accept(&mut self.scanner);
        Some(command)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::quit::Quit;
    #[test]
    fn test_command() {
        let commands =
            br#"get "toot"   put "toot" "data" delete "toot"  clear  unknown auth quit "tenant 1""#;
        let mut scanner = Scanner::new(commands);

        let command = Command::accept(&mut scanner).expect("Unable to parse command");
        assert_eq!(command, Command::Get(Get { key: b"toot" }));

        let command = Command::accept(&mut scanner).expect("Unable to parse command");
        assert_eq!(
            command,
            Command::Put(Put {
                key: b"toot",
                value: b"data"
            })
        );

        let command = Command::accept(&mut scanner).expect("Unable to parse command");
        assert_eq!(command, Command::Delete(Delete { key: b"toot" }));

        let command = Command::accept(&mut scanner).expect("Unable to parse command");
        assert_eq!(command, Command::Clear(Clear));

        let command = Command::accept(&mut scanner).expect("Unable to parse command");
        assert_eq!(command, Command::Unknown(Unknown));

        let command = Command::accept(&mut scanner).expect("Unable to parse command");
        assert_eq!(command, Command::Auth(Auth { tenant: "tenant 1" }));

        assert!(scanner.is_empty());
    }

    #[test]
    fn test_command_iterator() {
        let commands =
            br#"get "toot"   put "toot" "data" delete "toot"  clear  stats quit unknown auth "tenant 1""#;
        let command = Commands::new(commands);
        let commands = command.collect::<Vec<_>>();
        assert_eq!(commands.len(), 8);
        assert!(matches!(
            commands[0],
            Ok(Command::Get(Get { key: b"toot" }))
        ));
        assert!(matches!(
            commands[1],
            Ok(Command::Put(Put {
                key: b"toot",
                value: b"data"
            }))
        ));
        assert!(matches!(
            commands[2],
            Ok(Command::Delete(Delete { key: b"toot" }))
        ));
        assert!(matches!(commands[3], Ok(Command::Clear(Clear))));
        assert!(matches!(commands[4], Ok(Command::Stats(Stats))));
        assert!(matches!(commands[5], Ok(Command::Quit(Quit))));
        assert!(matches!(commands[6], Ok(Command::Unknown(Unknown))));
        assert!(matches!(
            commands[7],
            Ok(Command::Auth(Auth { tenant: "tenant 1" }))
        ));
    }
}
