use crate::error::{ParseError, ParseErrorKind, Severity};
use crate::Item;
use paste::paste;

mod new {
    use std::slice::SliceIndex;

    use super::*;

    struct SliceSpan<'a, I, E=()> {
        span: &'a [I],
        offset: usize,
        len: usize,
        extra : E,
    }

    impl<'a, I> SliceSpan<'a, I> {
        pub fn new(span: &'a [I]) -> Self {
            Self {
                span,
                offset: 0,
                len: span.len(),
                extra: (),
            }
        }
    }

    impl<'a, I,E> SliceSpan<'a, I, E> {
        pub fn new_extra(span: &'a [I], extra: E) -> Self {
            Self {
                span,
                offset: 0,
                len: span.len(),
                extra
            }
        }
        fn get_range(&self) -> std::ops::Range<usize> {
            self.offset..self.offset + self.len
        }
    }

    // Do we have a span of spans?
    // impl<'a, I,X,E > SliceSpan<'a, I, E> 
    //     where
    //         I : SpanTrait<Item=Xkk>
    // {
    //     fn underlying_span(&self) -> I {
    //         let first = self.span.first();
    //         let last = self.span.last();
    //         panic!()
    //     }

    // }

    impl<'a, I> SpanTrait for SliceSpan<'a, I> {
        type Item = I;


        fn length(&self) -> usize {
            self.len
        }

        fn take(&self, n: usize) -> Result<Self, ParseErrorKind> {
            if n > self.length() {
                Err(ParseErrorKind::TookTooMany)
            } else {
                Ok(Self {
                    offset: self.offset + n,
                    len: self.len - n,
                    ..*self
                })
            }
        }

        fn drop(&self, n: usize) -> Result<Self, ParseErrorKind> {
            if n > self.length() {
                Err(ParseErrorKind::TookTooMany)
            } else {
                Ok(Self { len: n, ..*self })
            }
        }

        fn at(&self, index: usize) -> Option<&Self::Item> {
            self.span[self.get_range()].get(index)
        }

        fn original(&self) -> Self {
            Self {
                offset: 0,
                len: self.span.len(),
                ..*self
            }
        }

    }

    pub trait SpanTrait: Sized {
        type Item;

        fn length(&self) -> usize;
        fn take(&self, len: usize) -> Result<Self, ParseErrorKind>;
        fn drop(&self, n: usize) -> Result<Self, ParseErrorKind>;
        fn at(&self, index: usize) -> Option<&Self::Item>;

        fn original(&self) -> Self;

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

        fn split(&self, n: usize) -> Result<(Self, Self), ParseErrorKind> {
            if n > self.length() {
                Err(ParseErrorKind::IllegalSplitIndex)
            } else {
                let rest = self.drop(n)?;
                let matched = self.take(n)?;
                Ok((rest, matched))
            }
        }

    }
}

pub trait Parser<I, O, E>: Clone + Copy
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

    fn take(&self, pos: usize) -> Result<Self, E> {
        let (rest, matched) = self.split_at(pos)?;
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
