use crate::span::Span;
use thiserror::Error;

#[derive(Error,Clone, Debug, PartialEq, )]
pub enum ParseErrorKind {
    #[error("not enough to take")]
    TookTooMany,
    #[error("not enough to skip")]
    SkippedTooMany,
    #[error("not enough to split")]
    IllegalSplitIndex,
    #[error("No match")]
    NoMatch,
    #[error("Match with not")]
    MatchedWithNot,
    #[error("Needed one or more matches")]
    NeededOneOrMore,
    #[error("Missing wrap terminator")]
    MissingWrapTerminator,
    #[error("Until never matched")]
    UntilNotMatched,
    #[error("Unconsumed input")]
    UnconsumedInput,
}

pub type PResult<'a, I, O = Span<'a, I>> = Result<(Span<'a, I>, O), ParseErrorKind>;

#[derive(Debug, PartialEq, Clone,Copy)]
pub enum Severity {
    Error,
    Fatal,
}

pub trait ParseError<I>: Sized + Clone 
where 
    I : Clone,
{
    fn from_error_kind(input: I, kind: ParseErrorKind, sev: Severity) -> Self;

    fn from_fatal_error(input: I, kind: ParseErrorKind)  -> Self {
        Self::from_error_kind(input, kind, Severity::Fatal)
    }

    fn from_error(input: I, kind: ParseErrorKind)  -> Self {
        Self::from_error_kind(input, kind, Severity::Error)
    }

    fn change_kind(self, kind: ParseErrorKind) -> Self;
    fn set_severity(self, sev: Severity)-> Self;
    fn severity(&self) -> Severity;
    fn append(input: I, kind: ParseErrorKind, other: Self) -> Self;

    fn is_fatal(&self) -> bool {
        self.severity() == Severity::Fatal
    }

}



