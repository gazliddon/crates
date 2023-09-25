use crate::error::{PResult, ParseError, ParseErrorKind, Severity};

use crate::traits::*;

pub trait Item  : Clone {
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
pub struct Span<'a, I >
where
    I: Item,
{
    position: usize, // index into parent doc
    len : usize,
    x_span: &'a [I],   // this span
}

impl<'a, I> Span<'a, I>
where
    I: Item,
{
    pub fn get(&self, idx: usize) -> Option<&I> {
        self.x_span.get(idx)
    }

    pub fn get_range(&self) -> std::ops::Range<usize> {
        self.position..self.position + self.len()
    }
    pub fn get_item_at_abs_position_sat(&self, pos : usize) -> Option<&I> {
        assert!(!self.x_span.is_empty());
        let pos = std::cmp::min(pos,self.x_span.len()-1);
        self.x_span.get(pos)
    }

    pub fn from_slice(text: &'a [I]) -> Self {
        Self::new(text, 0, text.len())
    }

    pub fn as_slice(&self) -> &[I] {
        &self.x_span[self.position..self.position+self.len]
    }

    pub fn new(x_span: &'a [I], position: usize, len: usize) -> Self {
        Self { position, len, x_span,  
    }}

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

    pub fn take(&self, n: usize) -> Result<Self, ParseErrorKind> {
        if n > self.len() {
            Err(ParseErrorKind::TookTooMany)
        } else {
            Ok(Self::new(self.x_span, self.position , n))
        }
    }

    pub fn drop(&self, n: usize) -> Result<Self, ParseErrorKind> {
        if n > self.len() {
            Err(ParseErrorKind::SkippedTooMany)
        } else {
            Ok(Self::new(self.x_span, self.position + n, self.len - n))
        }
    }

    pub fn split(&self, n: usize) -> Result<(Self, Self), ParseErrorKind> {
        if n > self.len() {
            Err(ParseErrorKind::IllegalSplitIndex)
        } else {
            let rest = self.drop(n)?;
            let matched = self.take(n)?;
            Ok((rest,matched))
        }
    }

    fn match_token(&'a self, other: &'a [<I as Item>::Kind]) -> PResult<'a, I> {
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

impl<'a, I, E> Splitter<E> for Span<'a, I>
where
    I: Item,
    E: ParseError<Span<'a, I>>,
{
    fn split_at(&self, pos: usize) -> Result<(Self, Self), E> {

        self.split(pos)
            .map_err(|e| ParseError::from_error(self.clone(), e, ))
    }
}

impl<I> Collection for Span<'_, I>
where
    I: Item,
{
    type Item = I;

    fn at(&self, index: usize) -> Option<&Self::Item> {
        self.as_slice().get(index)
    }

    fn length(&self) -> usize {
        self.len()
    }
}

// impl Item for char {
//     type Kind = char;

//     fn get_kind(&self) -> Self::Kind {
//         *self
//     }
// }

// impl Item for u8 {
//     type Kind = u8;
//     fn get_kind(&self) -> Self::Kind {
//         *self
//     }
// }
