#![allow(unused)]

use unraveler::{
    alt, eof, kind, kind_if, many0, many1, map_res, none_of_kinds, one_of_kinds, pair, peek,
    sep_list0, tag, terminated, tuple, value, verify, wrapped, Collection, Item, ParseError,
    ParseErrorKind, Parser, Severity,
};

type Span<'a> = unraveler::Span<'a, Token>;

#[derive(Debug, Clone, Copy)]
struct NewError {
    severity: Severity,
}

impl ParseError<Span<'_>> for NewError {
    fn from_error_kind(input: Span, kind: ParseErrorKind, sev: Severity) -> Self {
        let _ = (input, kind);
        NewError { severity: sev }
    }

    fn append(input: Span, kind: ParseErrorKind, other: Self) -> Self {
        other
    }

    fn change_kind(self, kind: ParseErrorKind) -> Self {
        self
    }

    fn set_severity(self, sev: Severity) -> Self {
        Self { severity: sev }
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn merge(self, other: Self) -> Self {
        if other.severity == Severity::Fatal {
            other
        } else {
            self
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum TokenKind {
    A,
    B,
    C,
}

impl Item for TokenKind {
    type Kind = TokenKind;

    fn get_kind(&self) -> TokenKind {
        *self
    }
}

#[derive(Copy, Clone, Debug)]
struct Token {
    kind: TokenKind,
    x: usize,
}

impl Token {
    pub fn new(kind: TokenKind) -> Self {
        Self { kind, x: 255 }
    }
}

impl Item for Token {
    type Kind = TokenKind;

    fn get_kind(&self) -> TokenKind {
        self.kind
    }
}

fn to_kinds(sp: Span) -> Vec<TokenKind> {
    sp.kinds_iter().collect()
}

fn to_tokens(kinds: &[TokenKind]) -> Vec<Token> {
    kinds.iter().cloned().map(Token::new).collect()
}

#[test]
fn test_norm() {
    use TokenKind::*;
    let doc = to_tokens(&[A, B, B, A, C, A, B]);
    let input = Span::from_slice(&doc, ());

    let (rest, (left, right)) = test_fn(input).unwrap();

    println!("0: {:?} {:?}", to_kinds(input), input.get_range());
    println!("1: {:?} {:?}", to_kinds(left), left.get_range());
    println!("2: {:?} {:?}", to_kinds(right), right.get_range());
    println!("r: {:?} {:?}", to_kinds(rest), rest.get_range());
    // assert!(false)
}

#[test]
fn test_tuple() -> Result<(), NewError> {
    use TokenKind::*;
    let doc = to_tokens(&[A, B, B, A, A, A, B]);
    let input = Span::from_slice(&doc, ());

    let (rest, (a, b, c)) = tuple((tag([A, B]), tag([B]), many0(tag([A]))))(input)?;
    assert_eq!(to_kinds(a), [A, B]);
    assert_eq!(to_kinds(b), [B]);
    let c: Vec<_> = c.into_iter().flat_map(to_kinds).collect();
    println!("VEC: {:?}", c);

    Ok(())
}

#[test]
fn test_alt() -> Result<(), NewError> {
    use TokenKind::*;
    let doc = to_tokens(&[B, A, A, A, A, A, B]);
    let input = Span::from_slice(&doc, ());

    let (rest, matched) = alt((tag([B, A]), tag([A, A])))(input)?;

    println!("ret: {:?}", to_kinds(matched));
    // assert!(false);

    Ok(())
}

#[test]
fn generic_conversion_and_kind_predicates() -> Result<(), NewError> {
    use TokenKind::*;
    let doc = to_tokens(&[A, B]);
    let input = Span::from_slice(&doc, ());

    let (rest, converted) = map_res(kind_if(|kind| *kind == A), |_| Ok::<_, NewError>(42))(input)?;
    assert_eq!(converted, 42);
    assert_eq!(rest.length(), 1);

    let (rest, replacement) = value("matched", kind(B))(rest)?;
    assert_eq!(replacement, "matched");
    assert!(rest.is_empty());
    Ok(())
}

#[test]
fn lookahead_validation_and_terminators_are_generic() -> Result<(), NewError> {
    use TokenKind::*;
    let doc = to_tokens(&[A, B, C]);
    let input = Span::from_slice(&doc, ());

    let (rest, looked) = peek(kind(A))(input)?;
    assert_eq!(looked.length(), 1);
    assert_eq!(rest.length(), 3);

    let (rest, matched) = verify(one_of_kinds([A, B]), |span: &Span| {
        span.first().unwrap().kind == A
    })(rest)?;
    assert_eq!(matched.length(), 1);
    assert_eq!(rest.length(), 2);

    let (rest, matched) = terminated(kind(B), kind(C))(rest)?;
    assert_eq!(matched.length(), 1);
    assert!(rest.is_empty());
    eof().parse(rest)?;

    let doc = to_tokens(&[A]);
    let input = Span::from_slice(&doc, ());
    let (_, matched) = none_of_kinds([B, C])(input)?;
    assert_eq!(matched.length(), 1);
    Ok(())
}

#[test]
fn test_many() -> Result<(), NewError> {
    use TokenKind::*;

    let doc = to_tokens(&[B, A, A, A, A, A, B]);

    let input = Span::from_slice(&doc, ());

    let res = pair(tag([B]), many0(tag([A])))(input);

    let (rest, (open, v)) = res?;

    let v: Vec<_> = v.iter().flat_map(|i| to_kinds(*i)).collect();

    println!("o: {:?}", to_kinds(open));
    println!("v: {:?}", v);
    println!("r: {:?} {:?}", to_kinds(rest), rest.get_range());

    assert_eq!(to_kinds(open), [B]);
    assert_eq!(v, [A, A, A, A, A]);
    assert_eq!(to_kinds(rest), [B]);

    Ok(())
}

#[test]
fn many1_returns_the_remaining_input() -> Result<(), NewError> {
    use TokenKind::*;
    let doc = to_tokens(&[A, A, B]);
    let input = Span::from_slice(&doc, ());

    let (rest, matched) = many1(tag([A]))(input)?;
    assert_eq!(matched.len(), 2);
    assert_eq!(to_kinds(rest), [B]);
    Ok(())
}

#[test]
fn wrapped_consumes_the_closing_delimiter() -> Result<(), NewError> {
    use TokenKind::*;
    let doc = to_tokens(&[A, B, C]);
    let input = Span::from_slice(&doc, ());

    let (rest, matched) = wrapped([A], tag([B]), [C])(input)?;
    assert_eq!(to_kinds(matched), [B]);
    assert!(rest.is_empty());
    Ok(())
}

#[test]
fn kind_matches_a_single_token_kind() -> Result<(), NewError> {
    use TokenKind::*;
    let doc = to_tokens(&[A, B]);
    let input = Span::from_slice(&doc, ());

    let (rest, matched) = kind(A)(input)?;
    assert_eq!(to_kinds(matched), [A]);
    assert_eq!(to_kinds(rest), [B]);
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PositionedError {
    offset: usize,
    severity: Severity,
}

impl ParseError<Span<'_>> for PositionedError {
    fn from_error_kind(input: Span, _kind: ParseErrorKind, severity: Severity) -> Self {
        Self {
            offset: input.offset(),
            severity,
        }
    }

    fn append(input: Span, _kind: ParseErrorKind, other: Self) -> Self {
        Self {
            offset: input.offset(),
            severity: other.severity,
        }
    }

    fn change_kind(self, _kind: ParseErrorKind) -> Self {
        self
    }

    fn set_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn merge(self, other: Self) -> Self {
        if other.offset > self.offset {
            other
        } else {
            self
        }
    }
}

#[test]
fn alt_returns_the_furthest_branch_error() {
    let doc = to_tokens(&[TokenKind::A, TokenKind::B]);
    let input = Span::from_slice(&doc, ());
    let first = |input: Span| {
        Err::<(Span, ()), _>(PositionedError {
            offset: input.offset(),
            severity: Severity::Error,
        })
    };
    let second = |input: Span| {
        Err::<(Span, ()), _>(PositionedError {
            offset: input.offset() + 1,
            severity: Severity::Error,
        })
    };

    let error = alt((first, second))(input).unwrap_err();
    assert_eq!(error.offset, 1);
}

#[derive(Clone)]
struct NonCopyInput<'a> {
    span: Span<'a>,
    marker: String,
}

impl Collection for NonCopyInput<'_> {
    type Item = Token;

    fn at(&self, index: usize) -> Option<&Self::Item> {
        self.span.at(index)
    }

    fn length(&self) -> usize {
        self.span.length()
    }
}

#[derive(Debug, Clone, Copy)]
struct NonCopyError {
    severity: Severity,
}

impl ParseError<NonCopyInput<'_>> for NonCopyError {
    fn from_error_kind(_input: NonCopyInput, _kind: ParseErrorKind, severity: Severity) -> Self {
        Self { severity }
    }

    fn append(_input: NonCopyInput, _kind: ParseErrorKind, other: Self) -> Self {
        other
    }

    fn change_kind(self, _kind: ParseErrorKind) -> Self {
        self
    }

    fn set_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }

    fn severity(&self) -> Severity {
        self.severity
    }

    fn merge(self, other: Self) -> Self {
        if other.severity == Severity::Fatal {
            other
        } else {
            self
        }
    }
}

impl unraveler::Splitter<NonCopyError> for NonCopyInput<'_> {
    fn split_at(&self, position: usize) -> Result<(Self, Self), NonCopyError> {
        let (rest, matched) = self.span.split(position).map_err(|_| NonCopyError {
            severity: Severity::Error,
        })?;
        Ok((
            Self {
                span: rest,
                marker: self.marker.clone(),
            },
            Self {
                span: matched,
                marker: self.marker.clone(),
            },
        ))
    }
}

#[test]
fn parsers_accept_non_copy_inputs() -> Result<(), NonCopyError> {
    let doc = to_tokens(&[TokenKind::A, TokenKind::B]);
    let input = NonCopyInput {
        span: Span::from_slice(&doc, ()),
        marker: String::from("non-copy"),
    };

    let (rest, (a, b)) = pair(kind(TokenKind::A), kind(TokenKind::B))(input)?;
    assert_eq!(a.length(), 1);
    assert_eq!(b.length(), 1);
    assert!(rest.is_empty());
    Ok(())
}

#[test]
fn repetition_rejects_a_parser_that_makes_no_progress() {
    use TokenKind::*;
    let doc = to_tokens(&[A]);
    let input = Span::from_slice(&doc, ());

    let result = many0(no_progress)(input);
    assert_eq!(result.unwrap_err().severity(), Severity::Error);
}

fn no_progress<'a>(input: Span<'a>) -> Result<(Span<'a>, ()), NewError> {
    Ok((input, ()))
}

#[test]
fn optional_parser_does_not_swallow_fatal_errors() {
    let doc = to_tokens(&[TokenKind::A]);
    let input = Span::from_slice(&doc, ());
    let fatal = |_input: Span| {
        Err::<(Span, ()), _>(NewError {
            severity: Severity::Fatal,
        })
    };

    let result = unraveler::opt(fatal)(input);
    assert_eq!(result.unwrap_err().severity(), Severity::Fatal);
}

#[test]
fn separated_lists_accept_non_clone_parsers() -> Result<(), NewError> {
    use TokenKind::*;
    let doc = to_tokens(&[A, B, A]);
    let input = Span::from_slice(&doc, ());

    let (rest, values) = sep_list0(NonCloneParser, tag([B]))(input)?;
    assert_eq!(values.len(), 2);
    assert!(rest.is_empty());
    Ok(())
}

struct NonCloneParser;

impl Parser<Span<'_>, TokenKind, NewError> for NonCloneParser {
    fn parse<'a>(&mut self, input: Span<'a>) -> Result<(Span<'a>, TokenKind), NewError> {
        let (rest, _) = tag([TokenKind::A])(input)?;
        Ok((rest, TokenKind::A))
    }
}

fn test_fn(input: Span) -> Result<(Span, (Span, Span)), NewError> {
    use TokenKind::*;

    let (rest, (matched, x)) = pair(tag(&[A, B]), tag(&[B, A]))(input)?;

    Ok((rest, (matched, x)))
}
