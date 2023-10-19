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
    E: Copy + Clone,
{
    pub fn new_extra(x_span: &'a [I], position: usize, len: usize, extra: E) -> Self {
        Self {
            position,
            len,
            x_span,
            extra,
        }
    }

    pub fn with_extra(self, extra : E) -> Self {
        Self {
            extra,
            ..self
        }
    }
}

impl<'a, I,E> Span<'a, I,E>
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
    pub fn get(&self, idx: usize) -> Option<&I> {
        self.as_slice().get(idx)
    }

    pub fn first(&self) -> Option<&I> {
        self.as_slice().first()
    }

    pub fn last(&self) -> Option<&I> {
        self.as_slice().last()
    }

    pub fn get_range(&self) -> std::ops::Range<usize> {
        self.position..self.position + self.len()
    }
    pub fn get_item_at_abs_position_sat(&self, pos: usize) -> Option<&I> {
        assert!(!self.x_span.is_empty());
        let pos = std::cmp::min(pos, self.x_span.len() - 1);
        self.x_span.get(pos)
    }

    pub fn from_slice(x_span: &'a [I], extra: XTRA) -> Self {
        Self {
            position: 0,
            len: x_span.len(),
            x_span, 
            extra
        }
    }

    pub fn as_slice(&self) -> &[I] {
        &self.x_span[self.position..self.position + self.len]
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.as_slice().is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &I> + '_ {
        self.as_slice().iter()
    }

    pub fn kinds_iter(&self) -> impl Iterator<Item = I::Kind> + '_ {
        self.iter().map(|i| i.get_kind())
    }

    pub fn take(&self, len: usize) -> Result<Self, ParseErrorKind> {
        if len > self.len() {
            Err(ParseErrorKind::TookTooMany)
        } else {
            let r = Self {
                len,
                ..self.clone()
            };
            Ok(r)
        }
    }

    pub fn drop(&self, n: usize) -> Result<Self, ParseErrorKind> {
        if n > self.len() {
            Err(ParseErrorKind::SkippedTooMany)
        } else {
            let r = Self {
                position: self.position + n,
                len : self.len -1,
                ..self.clone()
            };
            Ok(r)
        }
    }

    pub fn split(&self, n: usize) -> Result<(Self, Self), ParseErrorKind> {
        if n > self.len() {
            Err(ParseErrorKind::IllegalSplitIndex)
        } else {
            let rest = self.drop(n)?;
            let matched = self.take(n)?;
            Ok((rest, matched))
        }
    }

    fn match_token(&'a self, other: &'a [<I as Item>::Kind]) -> PResult<'a, I,XTRA> {
        if self.len() < other.len() {
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
}

impl<'a, I, E,XTRA> Splitter<E> for Span<'a, I, XTRA>
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

impl<I,XTRA> Collection for Span<'_, I,XTRA>
where
    I: Item,
    XTRA: Copy + Clone,
{
    type Item = I;

    fn at(&self, index: usize) -> Option<&Self::Item> {
        self.as_slice().get(index)
    }

    fn length(&self) -> usize {
        self.len()
    }
}
