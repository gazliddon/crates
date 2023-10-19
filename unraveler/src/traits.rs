use crate::error::{ParseError, ParseErrorKind, Severity};
use crate::Item;
use paste::paste;

pub trait Parser<I, O, E> : Clone + Copy
where
    I: Clone + Copy,
{
    fn parse(&mut self, i: I) -> Result<(I, O), E>;
}

impl<'a, I, O, E, F> Parser<I, O, E> for F
where
    I: Clone + Copy,
    F: FnMut(I) -> Result<(I, O), E> + 'a,
    F: Clone + Copy,
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
        let (rest, matched) = self.split_at(pos)?;
        Ok(rest)
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

    OTHER: Collection + Copy,
    <OTHER as Collection>::Item: Item + Copy,

    E: ParseError<SP>,
{
    fn tag(&self, other: OTHER) -> Result<(Self, Self), E> {
        if other.length() > self.length() {
            return Err(E::from_error(self.clone(), ParseErrorKind::NoMatch));
        }

        let mut index = 0;

        for i in 0..other.length() {
            let a = self.at(i).unwrap().get_kind();
            let b = other.at(i).unwrap().get_kind();

            if a != b {
                let err_pos = self.drop(i).unwrap_or_else(|_| panic!());
                return Err(E::from_error(err_pos, ParseErrorKind::NoMatch));
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
