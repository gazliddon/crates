use crate::error::{PResult, ParseError, ParseErrorKind, Severity};

use crate::traits::*;

pub trait Item: Clone {
    type Kind: Clone + PartialEq;

    fn is_same_kind<I>(&self, other: &I) -> bool
    where
        I: Item<Kind = Self::Kind>,
    {
        self.get_kind() == other.get_kind()
    }

    fn is_kind(&self, k: Self::Kind) -> bool {
        self.get_kind() == k
    }

    fn get_kind(&self) -> Self::Kind;
}

#[derive(Copy, Clone, Debug)]
pub struct Span<'a, I, E = ()>
where
    I: Item,
    E: Copy + Clone,
{
    position: usize, // index into parent doc
    len: usize,
    x_span: &'a [I], // this span
    extra: E,
}

impl<'a, I, E> Span<'a, I, E> 
where
    I: Item,
    E: Copy + Clone + std::default::Default,
{
    pub fn new(x_span: &'a [I], position: usize, len: usize) -> Self {
        Self {
            position,
            len,
            x_span,
            extra: Default::default(),
        }
    }
}

impl<'a, I, XTRA> Span<'a, I, XTRA>
where
    I: Item,
    XTRA: Copy + Clone,
{

    pub fn new_extra(x_span: &'a [I], position: usize, len: usize, extra: XTRA) -> Self {
        Self {
            position,
            len,
            x_span,
            extra,
        }
    }

    pub fn with_extra(self, extra: XTRA) -> Self {
        Self { extra, ..self }
    }

    pub fn extra(&self) -> &XTRA {
        &self.extra
    }

    pub fn from_slice(x_span: &'a [I], extra: XTRA) -> Self {
        Self {
            position: 0,
            len: x_span.len(),
            x_span,
            extra,
        }
    }

    //////////////////////////////////////////////////////////////////////////////// 
    // Needed in trait
    pub fn get_document(&self) -> &[I] {
        self.x_span
    }

    pub fn get_range(&self) -> std::ops::Range<usize> {
        let r = self.position..(self.position + self.length());
        r
    }

    pub fn as_slice(&self) -> &[I] {
        &self.x_span[self.get_range()]
    }


    pub fn take(&self, len: usize) -> Result<Self, ParseErrorKind> {
        if len > self.length() {
            Err(ParseErrorKind::TookTooMany)
        } else {
            Ok(Self {
                len,
                ..self.clone()
            })
        }
    }

    pub fn drop(&self, n: usize) -> Result<Self, ParseErrorKind> {
        if n > self.length() {
            Err(ParseErrorKind::SkippedTooMany)
        } else {
            Ok(Self {
                position: self.position + n,
                len: self.len - n,
                ..self.clone()
            })
        }
    }

    ////////////////////////////////////////////////////////////////////////////////
    // Kinds?
    pub fn kinds_iter(&self) -> impl Iterator<Item = I::Kind> + '_ {
        self.iter().map(|i| i.get_kind())
    }

    ////////////////////////////////////////////////////////////////////////////////
    pub fn split(&self, n: usize) -> Result<(Self, Self), ParseErrorKind> {
        if n > self.length() {
            Err(ParseErrorKind::IllegalSplitIndex)
        } else {
            let rest = self.drop(n)?;
            let matched = self.take(n)?;
            Ok((rest, matched))
        }
    }

    fn match_token(&'a self, other: &'a [<I as Item>::Kind]) -> PResult<'a, I, XTRA> {
        if self.length() < other.len() {
            Err(ParseErrorKind::NoMatch)
        } else {
            let it = self.iter().zip(other.iter());

            for (i, k) in it {
                if i.get_kind() != *k {
                    return Err(ParseErrorKind::NoMatch);
                }
            }

            self.split(other.len())
        }
    }

    fn match_kind(&self, k: I::Kind) -> bool {
        self.as_slice()
            .get(0)
            .map(|i| i.is_kind(i.get_kind()))
            .unwrap_or(false)
    }

    pub fn iter(&self) -> impl Iterator<Item = &I> + '_ {
        self.as_slice().iter()
    }

    pub fn offset(&self) -> usize {
        self.get_range().start
    }
}

impl<'a, I, E, XTRA> Splitter<E> for Span<'a, I, XTRA>
where
    I: Item,
    E: ParseError<Span<'a, I, XTRA>>,
    XTRA: Copy + Clone,
{
    fn split_at(&self, pos: usize) -> Result<(Self, Self), E> {
        self.split(pos)
            .map_err(|e| ParseError::from_error(self.clone(), e))
    }
}

impl<I, XTRA> Collection for Span<'_, I, XTRA>
where
    I: Item,
    XTRA: Copy + Clone,
{
    type Item = I;

    fn at(&self, index: usize) -> Option<&Self::Item> {
        self.as_slice().get(index)
    }

    fn length(&self) -> usize {
        self.len
    }
}
