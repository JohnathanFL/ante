//! parser/error.rs - Defines the ParseError type and the formatting shown
//! when printing this error to stderr.
use super::combinators::Input;
use crate::error::location::{Locatable, Location};
use crate::error::{Diagnostic, DiagnosticKind as D};
use crate::lexer::token::{LexerError, Token};
use crate::util::fmap;

#[derive(Debug)]
pub enum ParseError<'a> {
    /// A parsing error may not be fatal if it can be ignored because
    /// e.g. the parser is currently within an `or([...])` combinator that
    /// succeeds if any of the parsers in its array succeed.
    Fatal(Box<ParseError<'a>>),

    /// Expected any of the given tokens, but found... whatever is at the
    /// source Location instead
    Expected(Vec<Token>, Location<'a>),

    /// Failed while in the given parsing rule. E.g. "failed to parse a type".
    /// Due to backtracking this error is somewhat rare since the parser tends
    /// to backtrack trying to parse something else instead of failing in the
    /// rule that parsed the furthest. Proper usage of !<- (or `no_backtracking`)
    /// helps mediate this somewhat.
    InRule(&'static str, Location<'a>),

    /// Found a Token::Invalid issued by the lexer, containing some LexerError.
    /// These errors are always wrapped in a Fatal.
    LexerError(LexerError, Location<'a>),
}

pub type ParseResult<'local, 'cache, T> = Result<(Input<'local, 'cache>, T, Location<'cache>), ParseError<'cache>>;

impl<'a> Locatable<'a> for ParseError<'a> {
    fn locate(&self) -> Location<'a> {
        match self {
            ParseError::Fatal(error) => error.locate(),
            ParseError::Expected(_, location) => *location,
            ParseError::InRule(_, location) => *location,
            ParseError::LexerError(_, location) => *location,
        }
    }
}

impl<'a> ParseError<'a> {
    pub fn into_diagnostic(self) -> Diagnostic<'a> {
        match self {
            ParseError::Fatal(error) => error.into_diagnostic(),
            ParseError::Expected(tokens, location) => {
                let tokens = fmap(&tokens, ToString::to_string);
                Diagnostic::new(location, D::ParserExpected(tokens))
            },
            ParseError::InRule(rule, location) => Diagnostic::new(location, D::ParserErrorInRule(rule)),
            ParseError::LexerError(error, location) => Diagnostic::new(location, D::LexerError(error.to_string())),
        }
    }
}
