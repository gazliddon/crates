use super::{ParseError, Parser};
use paste::paste;

pub trait Tuple<I, O, E>
where
    I: Clone,
{
    fn tuple(&mut self, input: I) -> Result<(I, O), E>;
}

pub fn tuple<I: Clone, O, E: ParseError<I>, TUPLE: Tuple<I, O, E>>(
    mut l: TUPLE,
) -> impl FnMut(I) -> Result<(I, O), E> {
    move |i: I| l.tuple(i)
}

macro_rules! impl_tuple {
($($T:tt)*) => {
        paste! {
            impl<IX,EX,$($T,)*$([<O $T>],)*> Tuple<IX,($([<O $T>],)*),EX> for ($($T,)*)
            where
                $($T : Parser<IX,[<O $T>],EX>,)*
                EX : ParseError<IX>,
                IX : Clone,
            {
    fn tuple(&mut self, input: IX) -> Result<(IX, ($([<O $T>],)*)), EX> {
                let ($(ref mut [<$T:lower 1>],)*) = self;

                $(
                    let (input,[<out_$T:lower 1>]) = [<$T:lower 1>].parse(input)?;
                )*

                Ok(( input,($([<out_$T:lower 1>],)*) ))

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
