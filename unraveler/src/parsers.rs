use crate::error::*;
use crate::span::Item;
use crate::traits::*;

// use thin_vec::{thin_vec, ThinVec};

/// zero or more of the parser
pub fn many0<I, O, E, P>(mut p: P) -> impl FnMut(I) -> Result<(I, Vec<O>), E>
where
    I: Collection + Clone,
    P: Parser<I, O, E>,
    E: ParseError<I>,
{
    move |mut i: I| {
        let mut out = vec![];

        loop {
            // parse the first one
            let before = i.clone();
            let r = p.parse(i);

            match r {
                // All good, add matched to the vec and carry on parsing
                Ok((rest, matched)) => {
                    if rest.length() == before.length() {
                        return Err(E::from_error_kind(
                            before,
                            ParseErrorKind::NoProgress,
                            Severity::Error,
                        ));
                    }
                    i = rest;
                    out.push(matched)
                }

                // if we errored check to see if it's fatal
                Err(e) => {
                    if e.is_fatal() {
                        // Yes! So abort with an error
                        return Err(e);
                    } else {
                        // No! We've parsed as many as we can
                        return Ok((before, out));
                    }
                }
            }
        }
    }
}

/// Keep taking until the predicate is matched
pub fn many_until<I, O, PREDO, E, P, PRED>(
    mut p: P,
    mut pred: PRED,
) -> impl FnMut(I) -> Result<(I, Vec<O>), E>
where
    P: Parser<I, O, E>,
    PRED: Parser<I, PREDO, E>,
    I: Collection + Clone,
    E: ParseError<I>,
{
    move |mut i: I| {
        let mut out = vec![];

        loop {
            if i.length() == 0 {
                return Err(E::from_error_kind(
                    i,
                    ParseErrorKind::UntilNotMatched,
                    Severity::Error,
                ));
            }

            // Have we hit the predicate?
            let before = i.clone();
            let r = pred.parse(i.clone());

            match r {
                Ok((_rest, _)) => return Ok((before, out)),
                Err(e) => {
                    if e.is_fatal() {
                        return Err(e);
                    }
                }
            }

            let before = i.clone();
            let (rest, matched) = p.parse(i)?;
            if rest.length() == before.length() {
                return Err(E::from_error_kind(
                    before,
                    ParseErrorKind::NoProgress,
                    Severity::Error,
                ));
            }
            i = rest;
            out.push(matched)
        }
    }
}

/// One or more of the parser
pub fn many1<I, O, E, P>(mut p: P) -> impl FnMut(I) -> Result<(I, Vec<O>), E>
where
    P: Parser<I, O, E>,
    I: Collection + Clone,
    E: ParseError<I>,
{
    move |i: I| {
        let (mut i, x) = p.parse(i)?;
        let mut out = vec![x];
        loop {
            let before = i.clone();
            match p.parse(i) {
                Ok((rest, matched)) => {
                    if rest.length() == before.length() {
                        return Err(E::from_error_kind(
                            before,
                            ParseErrorKind::NoProgress,
                            Severity::Error,
                        ));
                    }
                    i = rest;
                    out.push(matched);
                }
                Err(e) if e.is_fatal() => return Err(e),
                Err(_) => return Ok((before, out)),
            }
        }
    }
}

/// Checks for the first parser and then the second parser
/// but doesn't capture first parser
pub fn preceded<I, O1, O2, P1, P2, E>(
    mut first: P1,
    mut second: P2,
) -> impl FnMut(I) -> Result<(I, O2), E>
where
    I: Clone,
    P1: Parser<I, O1, E>,
    P2: Parser<I, O2, E>,
    E: ParseError<I>,
{
    move |rest: I| {
        let (rest, _) = first.parse(rest)?;
        let (rest, matched_2) = second.parse(rest)?;
        Ok((rest, matched_2))
    }
}

/// Checks for the first parser and then the second parser
/// but doesn't capture the second
pub fn succeeded<I, O1, O2, P1, P2, E>(
    mut first: P1,
    mut second: P2,
) -> impl FnMut(I) -> Result<(I, O1), E>
where
    I: Clone,
    P1: Parser<I, O1, E>,
    P2: Parser<I, O2, E>,
    E: ParseError<I>,
{
    move |rest: I| {
        let (rest, matched_1) = first.parse(rest)?;
        let (rest, _) = second.parse(rest)?;
        Ok((rest, matched_1))
    }
}

pub fn pair<I, O1, O2, P1, P2, E>(
    mut first: P1,
    mut second: P2,
) -> impl FnMut(I) -> Result<(I, (O1, O2)), E>
where
    I: Clone,
    P1: Parser<I, O1, E>,
    P2: Parser<I, O2, E>,
    E: ParseError<I>,
{
    move |rest: I| {
        let (rest, matched_1) = first.parse(rest)?;
        let (rest, matched_2) = second.parse(rest)?;
        Ok((rest, (matched_1, matched_2)))
    }
}

/// Parse an optional value
pub fn opt<I, O, E, P>(mut first: P) -> impl FnMut(I) -> Result<(I, Option<O>), E>
where
    P: Parser<I, O, E>,
    E: ParseError<I>,
    I: Clone,
{
    move |input: I| match first.parse(input.clone()) {
        Ok((r, m)) => Ok((r, Some(m))),
        Err(e) if e.is_fatal() => Err(e),
        Err(_) => Ok((input, None)),
    }
}

/// Isn't this parser
pub fn not<I, O, E, P>(mut first: P) -> impl FnMut(I) -> Result<(I, I), E>
where
    I: Clone,
    P: Parser<I, O, E>,
    E: ParseError<I> + std::fmt::Debug,
{
    move |input: I| {
        let ret = first.parse(input.clone());

        match ret {
            Ok(_) => Err(E::from_error_kind(
                input,
                ParseErrorKind::NoMatch,
                Severity::Error,
            )),
            Err(e) if e.is_fatal() => Err(e),
            Err(_) => Ok((input.clone(), input)),
        }
    }
}

/// Make sure this parser runs to the end of input
pub fn all<I, O, E, P>(mut first: P) -> impl FnMut(I) -> Result<(I, O), E>
where
    I: Collection + Clone,
    P: Parser<I, O, E>,
    E: ParseError<I> + std::fmt::Debug,
{
    move |input: I| {
        let ret = first.parse(input.clone());
        match ret {
            Ok((rest, matched)) => {
                if !rest.is_empty() {
                    Err(E::from_error_kind(
                        input.clone(),
                        ParseErrorKind::UnconsumedInput,
                        Severity::Error,
                    ))
                } else {
                    Ok((rest, matched))
                }
            }
            Err(e) => Err(e.set_severity(Severity::Fatal)),
        }
    }
}

/// Parse has to succeed
pub fn cut<I, O, E, P>(mut first: P) -> impl FnMut(I) -> Result<(I, O), E>
where
    I: Clone,
    P: Parser<I, O, E>,
    E: ParseError<I> + std::fmt::Debug,
{
    move |input: I| {
        let ret = first.parse(input);
        match ret {
            Ok(r) => Ok(r),
            Err(e) => Err(e.set_severity(Severity::Fatal)),
        }
    }
}

pub fn sep_list0<I, O1, OS, P1, PS, E>(
    mut first: P1,
    mut sep: PS,
) -> impl FnMut(I) -> Result<(I, Vec<O1>), E>
where
    I: Collection + Clone,
    P1: Parser<I, O1, E>,
    PS: Parser<I, OS, E>,
    E: ParseError<I>,
{
    move |input: I| {
        let (mut rest, first_item) = match first.parse(input.clone()) {
            Ok(value) => value,
            Err(e) if e.is_fatal() => return Err(e),
            Err(_) => return Ok((input, vec![])),
        };
        let mut ret = vec![first_item];

        loop {
            let before_separator = rest.clone();
            let after_separator = match sep.parse(rest.clone()) {
                Ok((rest, _)) => rest,
                Err(e) if e.is_fatal() => return Err(e),
                Err(_) => return Ok((before_separator, ret)),
            };
            match first.parse(after_separator) {
                Ok((after_item, item)) => {
                    if after_item.length() == before_separator.length() {
                        return Err(E::from_error_kind(
                            before_separator,
                            ParseErrorKind::NoProgress,
                            Severity::Error,
                        ));
                    }
                    rest = after_item;
                    ret.push(item);
                }
                Err(e) if e.is_fatal() => return Err(e),
                Err(_) => return Ok((before_separator, ret)),
            }
        }
    }
}

pub fn sep_list<I, O1, OS, P1, PS, E>(
    mut first: P1,
    mut sep: PS,
) -> impl FnMut(I) -> Result<(I, Vec<O1>), E>
where
    I: Collection + Clone,
    P1: Parser<I, O1, E>,
    PS: Parser<I, OS, E>,
    E: ParseError<I>,
{
    move |input: I| {
        let (mut rest, first_item) = first.parse(input.clone())?;
        let mut ret = vec![first_item];

        loop {
            let before_separator = rest.clone();
            let after_separator = match sep.parse(rest.clone()) {
                Ok((rest, _)) => rest,
                Err(e) if e.is_fatal() => return Err(e),
                Err(_) => return Ok((before_separator, ret)),
            };
            match first.parse(after_separator) {
                Ok((after_item, item)) => {
                    if after_item.length() == before_separator.length() {
                        return Err(E::from_error_kind(
                            before_separator,
                            ParseErrorKind::NoProgress,
                            Severity::Error,
                        ));
                    }
                    rest = after_item;
                    ret.push(item);
                }
                Err(e) if e.is_fatal() => return Err(e),
                Err(_) => return Ok((before_separator, ret)),
            }
        }
    }
}

pub fn sep_pair<I, O1, O2, OS, P1, P2, PS, E>(
    mut first: P1,
    mut sep: PS,
    mut second: P2,
) -> impl FnMut(I) -> Result<(I, (O1, O2)), E>
where
    I: Clone,
    P1: Parser<I, O1, E>,
    P2: Parser<I, O2, E>,
    PS: Parser<I, OS, E>,
    E: ParseError<I>,
{
    move |input: I| {
        let (rest, matched_1) = first.parse(input)?;
        let (rest, _) = sep.parse(rest)?;
        let (rest, matched_2) = second.parse(rest)?;
        Ok((rest, (matched_1, matched_2)))
    }
}

/// Parse a value wrapped by a pair of other values
/// Good for parsing parenthisied values
pub fn wrapped<SP, OTHER, E, P, O>(
    open: OTHER,
    mut p: P,
    close: OTHER,
) -> impl FnMut(SP) -> Result<(SP, O), E>
where
    SP: Collection + Splitter<E> + Clone,
    <SP as Collection>::Item: Item,
    <<SP as Collection>::Item as Item>::Kind:
        PartialEq<<<OTHER as Collection>::Item as Item>::Kind>,

    OTHER: Collection + Clone,
    <OTHER as Collection>::Item: Item,

    E: ParseError<SP>,
    P: Parser<SP, O, E>,
{
    move |rest: SP| {
        let (rest, _) = rest.tag(open.clone())?;
        let (rest, matched) = p.parse(rest)?;
        let (rest, _) = rest.tag(close.clone())?;
        Ok((rest, matched))
    }
}

/// Parse a value wrapped by a pair of other values
/// Good for parsing parenthisied values
/// Errors hard if failed pn closing term
pub fn wrapped_cut<SP, OTHER, E, P, O>(
    open: OTHER,
    mut p: P,
    close: OTHER,
) -> impl FnMut(SP) -> Result<(SP, O), E>
where
    SP: Collection + Splitter<E> + Clone,
    <SP as Collection>::Item: Item,
    <<SP as Collection>::Item as Item>::Kind:
        PartialEq<<<OTHER as Collection>::Item as Item>::Kind>,
    OTHER: Collection + Clone,
    <OTHER as Collection>::Item: Item,

    E: ParseError<SP> + std::fmt::Debug,
    P: Parser<SP, O, E>,
{
    move |rest: SP| {
        let (rest, _) = rest.tag(open.clone())?;
        let (rest, matched) = p.parse(rest)?;
        let (rest, _) = cut(tag(close.clone()))(rest)
            .map_err(|e| e.change_kind(ParseErrorKind::MissingWrapTerminator))?;
        Ok((rest, matched))
    }
}

/// Parse a value between an opening and closing delimiter, treating a
/// missing closing delimiter as fatal. This is the clearer public spelling of
/// [`wrapped_cut`]; the older name remains available for compatibility.
pub fn delimited<SP, OTHER, E, P, O>(
    open: OTHER,
    p: P,
    close: OTHER,
) -> impl FnMut(SP) -> Result<(SP, O), E>
where
    SP: Collection + Splitter<E> + Clone,
    <SP as Collection>::Item: Item,
    <<SP as Collection>::Item as Item>::Kind:
        PartialEq<<<OTHER as Collection>::Item as Item>::Kind>,
    OTHER: Collection + Clone,
    <OTHER as Collection>::Item: Item,
    E: ParseError<SP> + std::fmt::Debug,
    P: Parser<SP, O, E>,
{
    wrapped_cut(open, p, close)
}

/// Parse a value between two individual token kinds.
///
/// This is the convenient form for token streams where delimiters are kinds
/// rather than one-item collections. A missing closing delimiter is fatal,
/// matching [`delimited`].
pub fn delimited_kind<SP, E, P, O>(
    open: <<SP as Collection>::Item as Item>::Kind,
    mut p: P,
    close: <<SP as Collection>::Item as Item>::Kind,
) -> impl FnMut(SP) -> Result<(SP, O), E>
where
    SP: Collection + Splitter<E> + Clone,
    <SP as Collection>::Item: Item,
    E: ParseError<SP> + std::fmt::Debug,
    P: Parser<SP, O, E>,
{
    move |input| {
        let (rest, _) = kind(open.clone())(input)?;
        let (rest, matched) = p.parse(rest)?;
        let (rest, _) = cut(kind(close.clone()))(rest)
            .map_err(|e| e.change_kind(ParseErrorKind::MissingWrapTerminator))?;
        Ok((rest, matched))
    }
}

/// Matches a string of items
pub fn tag<SP, OTHER, E>(tag: OTHER) -> impl FnMut(SP) -> Result<(SP, SP), E>
where
    SP: Collection + Splitter<E> + Clone,
    <SP as Collection>::Item: Item,
    <<SP as Collection>::Item as Item>::Kind:
        PartialEq<<<OTHER as Collection>::Item as Item>::Kind>,

    OTHER: Collection + Clone,
    <OTHER as Collection>::Item: Item,
    E: ParseError<SP>,
{
    move |input: SP| {
        let (rest, matched) = input.tag(tag.clone())?;
        Ok((rest, matched))
    }
}

/// Match a sequence of token kinds and return the consumed span.
///
/// Unlike [`tag`], this does not require the kind type itself to implement
/// [`Collection`] or [`Item`]. Arrays and slices can be passed directly.
pub fn tag_kinds<SP, K, C, E>(expected: C) -> impl FnMut(SP) -> Result<(SP, SP), E>
where
    SP: Collection + Splitter<E> + Clone,
    <SP as Collection>::Item: Item,
    K: PartialEq<<<SP as Collection>::Item as Item>::Kind>,
    C: AsRef<[K]>,
    E: ParseError<SP>,
{
    move |input| {
        let expected = expected.as_ref();
        if expected.len() > input.length() {
            return Err(E::from_error(input, ParseErrorKind::NoMatch));
        }

        for (index, expected_kind) in expected.iter().enumerate() {
            let Some(item) = input.at(index) else {
                return Err(E::from_error(input, ParseErrorKind::NoMatch));
            };
            let actual = item.get_kind();
            if expected_kind != &actual {
                return Err(E::from_error(input, ParseErrorKind::NoMatch));
            }
        }

        input.split_at(expected.len())
    }
}

pub fn any<SP, E>() -> impl FnMut(SP) -> Result<(SP, SP), E>
where
    SP: Splitter<E>,
    E: ParseError<SP>,
{
    move |input: SP| input.split_at(1)
}

/// Match and return one item when the predicate accepts it.
pub fn match_item<SP, E>(
    pred: impl Fn(&SP::Item) -> bool,
) -> impl FnMut(SP) -> Result<(SP, SP::Item), E>
where
    <SP as Collection>::Item: Item,
    SP: Collection + Splitter<E> + Clone,
    E: ParseError<SP>,
{
    move |input: SP| {
        let (rest, matched) = input.split_at(1)?;

        let i = matched.at(0).unwrap();

        if pred(i) {
            Ok((rest, i.clone()))
        } else {
            Err(ParseError::from_error(input, ParseErrorKind::NoMatch))
        }
    }
}

/// Match one item by its [`Item::Kind`] and return the consumed span.
///
/// This is the convenient token-parser counterpart to [`tag`].  It lets a
/// consumer parse a single token kind without implementing `Parser` for every
/// token-kind enum in the host crate.
pub fn kind<SP, E>(
    expected: <<SP as Collection>::Item as Item>::Kind,
) -> impl FnMut(SP) -> Result<(SP, SP), E>
where
    SP: Collection + Splitter<E> + Clone,
    <SP as Collection>::Item: Item,
    E: ParseError<SP>,
{
    move |input: SP| {
        let Some(first) = input.first() else {
            return Err(E::from_error(input, ParseErrorKind::NoMatch));
        };

        if first.is_kind(expected.clone()) {
            input.split_at(1)
        } else {
            Err(E::from_error(input, ParseErrorKind::NoMatch))
        }
    }
}

/// Match one item when a predicate over its kind succeeds.
///
/// This is useful for consumers whose token kind enum contains payloads (for
/// example `Number(radix, value)`) and therefore cannot conveniently use
/// [`kind`] with one fixed value.
pub fn kind_if<SP, E, F>(mut predicate: F) -> impl FnMut(SP) -> Result<(SP, SP), E>
where
    SP: Collection + Splitter<E> + Clone,
    <SP as Collection>::Item: Item,
    F: FnMut(&<<SP as Collection>::Item as Item>::Kind) -> bool,
    E: ParseError<SP>,
{
    move |input: SP| {
        let Some(first) = input.first() else {
            return Err(E::from_error(input, ParseErrorKind::NoMatch));
        };
        if predicate(&first.get_kind()) {
            input.split_at(1)
        } else {
            Err(E::from_error(input, ParseErrorKind::NoMatch))
        }
    }
}

/// Match one item whose kind occurs in `expected`.
pub fn one_of_kinds<SP, E, C>(expected: C) -> impl FnMut(SP) -> Result<(SP, SP), E>
where
    SP: Collection + Splitter<E> + Clone,
    <SP as Collection>::Item: Item,
    C: AsRef<[<<SP as Collection>::Item as Item>::Kind]>,
    E: ParseError<SP>,
{
    let expected = expected.as_ref().to_vec();
    kind_if(move |kind| expected.iter().any(|candidate| candidate == kind))
}

/// Match one item whose kind does not occur in `excluded`.
pub fn none_of_kinds<SP, E, C>(excluded: C) -> impl FnMut(SP) -> Result<(SP, SP), E>
where
    SP: Collection + Splitter<E> + Clone,
    <SP as Collection>::Item: Item,
    C: AsRef<[<<SP as Collection>::Item as Item>::Kind]>,
    E: ParseError<SP>,
{
    let excluded = excluded.as_ref().to_vec();
    kind_if(move |kind| excluded.iter().all(|candidate| candidate != kind))
}

pub fn until<SP, E>(
    pred: impl Fn(<SP as Collection>::Item) -> bool,
) -> impl FnMut(SP) -> Result<(SP, SP), E>
where
    SP: Collection + Splitter<E> + Clone,
    <SP as Collection>::Item: PartialEq + Item,
    E: ParseError<SP>,
{
    move |input: SP| -> Result<(SP, SP), E> {
        if input.is_empty() {
            Ok((input.clone(), input.clone()))
        } else {
            for index in 0..input.length() {
                let k = input.at(index).unwrap();

                if pred(k.clone()) {
                    return input.split_at(index);
                }
            }

            Err(ParseError::from_error(
                input,
                ParseErrorKind::UntilNotMatched,
            ))
        }
    }
}

pub fn is_a<SP, C, E>(
    isa: C,
) -> impl FnMut(SP) -> Result<(SP, <<SP as Collection>::Item as Item>::Kind), E>
where
    SP: Collection + Splitter<E> + Clone,
    <SP as Collection>::Item: PartialEq + Item,
    C: Collection,
    <C as Collection>::Item: PartialEq + Copy + Item,
    <<SP as Collection>::Item as Item>::Kind: PartialEq<<<C as Collection>::Item as Item>::Kind>,
    E: ParseError<SP>,
{
    move |input: SP| -> Result<(SP, <<SP as Collection>::Item as Item>::Kind), E> {
        if input.is_empty() {
            Err(ParseError::from_error(input, ParseErrorKind::NoMatch))
        } else {
            // Get the item kind of the first the input span
            let k = input.at(0).map(|x| x.get_kind());

            // Go through the isa collection and see if we can find a match

            for i in 0..isa.length() {
                let ik = isa.at(i).map(|x| x.get_kind());

                match (k.clone(), ik) {
                    (Some(a), Some(b)) => {
                        if a == b {
                            let r = input.drop(1).map(|x| (x, a.clone())).map_err(|_| {
                                ParseError::from_error(input, ParseErrorKind::NoMatch)
                            });
                            return r;
                        }
                    }
                    _ => return Err(ParseError::from_error(input, ParseErrorKind::NoMatch)),
                }
            }

            Err(ParseError::from_error(input, ParseErrorKind::NoMatch))
        }
    }
}

pub fn map<I, E, P, M, O, XO>(mut p: P, mut mapper: M) -> impl FnMut(I) -> Result<(I, O), E>
where
    P: FnMut(I) -> Result<(I, XO), E>,
    M: FnMut(XO) -> O,
    I: Clone,
    E: ParseError<I>,
{
    move |i: I| p.parse(i).map(|(r, m)| (r, mapper(m)))
}

/// Parse a value and convert it while retaining parser error handling.
///
/// The conversion error is deliberately the consumer's parser error type;
/// Unraveler does not prescribe how conversion failures should be reported.
pub fn map_res<I, E, P, M, O, XO>(mut p: P, mut mapper: M) -> impl FnMut(I) -> Result<(I, XO), E>
where
    P: Parser<I, O, E>,
    M: FnMut(O) -> Result<XO, E>,
    I: Clone,
    E: ParseError<I>,
{
    move |i: I| {
        let (rest, matched) = p.parse(i)?;
        Ok((rest, mapper(matched)?))
    }
}

/// Parse and discard the matched value, returning a caller-supplied value.
pub fn value<I, E, P, O, XO>(value: XO, mut p: P) -> impl FnMut(I) -> Result<(I, XO), E>
where
    P: Parser<I, O, E>,
    XO: Clone,
    I: Clone,
    E: ParseError<I>,
{
    move |i: I| p.parse(i).map(|(rest, _)| (rest, value.clone()))
}

/// Run a parser without consuming its input.
pub fn peek<I, E, P, O>(mut p: P) -> impl FnMut(I) -> Result<(I, O), E>
where
    P: Parser<I, O, E>,
    I: Clone,
    E: ParseError<I>,
{
    move |input: I| {
        let (_, matched) = p.parse(input.clone())?;
        Ok((input, matched))
    }
}

/// Parse a value and reject it when the validation predicate fails.
pub fn verify<I, E, P, O, F>(mut p: P, mut predicate: F) -> impl FnMut(I) -> Result<(I, O), E>
where
    P: Parser<I, O, E>,
    F: FnMut(&O) -> bool,
    I: Clone,
    E: ParseError<I>,
{
    move |input: I| {
        let (rest, matched) = p.parse(input.clone())?;
        if predicate(&matched) {
            Ok((rest, matched))
        } else {
            Err(E::from_error(input, ParseErrorKind::NoMatch))
        }
    }
}

/// Succeed only when the parser input has been fully consumed.
pub fn eof<I, E>() -> impl FnMut(I) -> Result<(I, ()), E>
where
    I: Collection + Clone,
    E: ParseError<I>,
{
    move |input: I| {
        if input.is_empty() {
            Ok((input, ()))
        } else {
            Err(E::from_error(input, ParseErrorKind::UnconsumedInput))
        }
    }
}

/// Parse `first`, then `terminator`, returning only `first`'s value.
pub fn terminated<I, E, P, T, O, OT>(
    mut first: P,
    mut terminator: T,
) -> impl FnMut(I) -> Result<(I, O), E>
where
    P: Parser<I, O, E>,
    T: Parser<I, OT, E>,
    I: Clone,
    E: ParseError<I>,
{
    move |input: I| {
        let (rest, matched) = first.parse(input)?;
        let (rest, _) = terminator.parse(rest)?;
        Ok((rest, matched))
    }
}

pub fn and_then<I, E, P, M, O, XO>(mut p: P, mut mapper: M) -> impl FnMut(I) -> Result<(I, XO), E>
where
    P: FnMut(I) -> Result<(I, O), E>,
    M: FnMut((I, O)) -> Result<(I, XO), E>,
    I: Clone,
    E: ParseError<I>,
{
    move |i: I| p.parse(i).and_then(&mut mapper)
}

pub fn match_span<P, I, O, E>(mut p: P) -> impl FnMut(I) -> Result<(I, (I, O)), E>
where
    I: Clone,
    P: Parser<I, O, E>,
    I: Splitter<E> + Collection,
    E: ParseError<I>,
{
    move |i| {
        let input = i.clone();
        let (rest, matched) = p.parse(i)?;
        let matched_len = input.length() - rest.length();
        let matched_span = input.take(matched_len)?;
        Ok((rest, (matched_span, matched)))
    }
}

/// Run a parser and return the exact input span it consumed alongside its
/// output. This is the preferred name for [`match_span`] in parser clients.
pub fn spanned<P, I, O, E>(p: P) -> impl FnMut(I) -> Result<(I, (I, O)), E>
where
    I: Clone + Splitter<E> + Collection,
    P: Parser<I, O, E>,
    E: ParseError<I>,
{
    match_span(p)
}
