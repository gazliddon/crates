use crate::error::{ParseError, ParseErrorKind};
use crate::Item;

pub trait Parser<I, O, E>
where
    I: Clone,
{
    fn parse(&mut self, i: I) -> Result<(I, O), E>;
}

impl<I, O, E, F> Parser<I, O, E> for F
where
    I: Clone,
    F: FnMut(I) -> Result<(I, O), E>,
{
    fn parse(&mut self, i: I) -> Result<(I, O), E> {
        self(i)
    }
}

pub trait Splitter<E>: Sized + Clone
where
    E: ParseError<Self>,
{
    fn split_at(&self, pos: usize) -> Result<(Self, Self), E>;

    fn drop(&self, pos: usize) -> Result<Self, E> {
        let (rest, _matched) = self.split_at(pos)?;
        Ok(rest)
    }

    fn take(&self, pos: usize) -> Result<Self, E> {
        let (_rest, matched) = self.split_at(pos)?;
        Ok(matched)
    }
}

pub trait Collection {
    type Item;

    fn at(&self, index: usize) -> Option<&Self::Item>;
    fn length(&self) -> usize;

    fn first(&self) -> Option<&Self::Item> {
        self.at(0)
    }

    fn last(&self) -> Option<&Self::Item> {
        if self.length() > 0 {
            self.at(self.length() - 1)
        } else {
            None
        }
    }
    fn is_empty(&self) -> bool {
        self.length() == 0
    }
}

pub trait Tag<OTHER, E>: Sized {
    fn tag(&self, other: OTHER) -> Result<(Self, Self), E>;
}

impl<SP, OTHER, E> Tag<OTHER, E> for SP
where
    SP: Collection + Splitter<E> + Clone,
    <SP as Collection>::Item: Item,
    <<SP as Collection>::Item as Item>::Kind:
        PartialEq<<<OTHER as Collection>::Item as Item>::Kind>,

    OTHER: Collection + Clone,
    <OTHER as Collection>::Item: Item,

    E: ParseError<SP>,
{
    fn tag(&self, other: OTHER) -> Result<(Self, Self), E> {
        if other.length() > self.length() {
            return Err(E::from_error(self.clone(), ParseErrorKind::NoMatch));
        }

        let mut index = 0;

        for i in 0..other.length() {
            let (Some(a), Some(b)) = (self.at(i), other.at(i)) else {
                return Err(E::from_error(self.clone(), ParseErrorKind::NoMatch));
            };
            let a = a.get_kind();
            let b = b.get_kind();

            if a != b {
                return Err(E::from_error(self.clone(), ParseErrorKind::NoMatch));
            } else {
                index += 1
            }
        }

        self.split_at(index)
    }
}

impl<X, const N: usize> Collection for &[X; N] {
    type Item = X;

    fn at(&self, index: usize) -> Option<&Self::Item> {
        self.get(index)
    }

    fn length(&self) -> usize {
        self.len()
    }
}

impl<X, const N: usize> Collection for [X; N] {
    type Item = X;

    fn at(&self, index: usize) -> Option<&Self::Item> {
        self.get(index)
    }

    fn length(&self) -> usize {
        self.len()
    }
}
