use super::{ParseError, ParseErrorKind, Parser};
use paste::paste;

pub trait Tuple<I, O, E>: Copy
where
    I: Clone + Copy,
{
    fn tuple(&mut self, input: I) -> Result<(I, O), E>;
}

pub fn tuple<I: Clone + Copy, O, E: ParseError<I>, TUPLE: Tuple<I, O, E>>(
    mut l: TUPLE,
) -> impl FnMut(I) -> Result<(I, O), E> + Copy {
    move |i: I| l.tuple(i)
}

macro_rules! impl_tuple {
($($T:tt)*) => {
        paste! {
            impl<IX,EX,$($T,)*$([<O $T>],)*> Tuple<IX,($([<O $T>],)*),EX> for ($($T,)*)
            where
                $($T : Parser<IX,[<O $T>],EX>,)*
                EX : ParseError<IX>,
                IX : Clone + Copy,
            {
    fn tuple(&mut self, input: IX) -> Result<(IX, ($([<O $T>],)*)), EX> {
                let ($(ref mut [<$T:lower 1>],)*) = self;

                let rest = input.clone();

                $(
                    let (rest,[<out_$T:lower 1>]) = [<$T:lower 1>].parse(rest.clone())?;
                )*;

                Ok(( rest,($([<out_$T:lower 1>],)*) ))

                // Err(EX::from_error_kind(&input,ParseErrorKind::NoMatch))
                }
            }
        }
    };
}

impl_tuple!(A B);
impl_tuple!(A B C);
impl_tuple!(A B C D);
impl_tuple!(A B C D E);
impl_tuple!(A B C D E F);
impl_tuple!(A B C D E F G);
impl_tuple!(A B C D E F G H);
impl_tuple!(A B C D E F G H I);
impl_tuple!(A B C D E F G H I J);
impl_tuple!(A B C D E F G H I J K);
impl_tuple!(A B C D E F G H I J K L);
