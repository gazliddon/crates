use crate::{ParseError, ParseErrorKind, Parser, Severity};
use paste::paste;

pub trait Alt<I, O, E> {
    fn choose(&mut self, input: I) -> Result<(I, O), E>;
}

pub fn alt<I: Clone, O, E: ParseError<I>, ALT: Alt<I, O, E>>(
    mut l: ALT,
) -> impl FnMut(I) -> Result<(I, O), E> {
    move |i: I| l.choose(i)
}

////////////////////////////////////////////////////////////////////////////////
// Macro to implement parse for alr
macro_rules! impl_alt_tuple {
($($T:tt)*) => {
        paste! {
            impl<IX,OX,EX,$($T,)*> Alt<IX,OX,EX> for ($($T,)*)
            where
                $($T : Parser<IX,OX,EX>,)*
                EX : ParseError<IX>,
                IX : Clone,
            {
    fn choose(&mut self, i: IX) -> Result<(IX, OX), EX> {
                let ($(ref mut [<$T:lower 1>],)*) = self;

                $(
                    let res = [<$T:lower 1>].parse(i.clone());

                    match &res  {
                        Ok(_) => return res,
                        Err(e) => if e.is_fatal() {
                            return res;
                        }
                    };
                )*;

                Err(EX::from_error(i,ParseErrorKind::NoMatch))
                }
            }
        }
    };
}

impl_alt_tuple!(A);
impl_alt_tuple!(A B);
impl_alt_tuple!(A B C);
impl_alt_tuple!(A B C D);
impl_alt_tuple!(A B C D E);
impl_alt_tuple!(A B C D E F);
impl_alt_tuple!(A B C D E F G);
impl_alt_tuple!(A B C D E F G H);
impl_alt_tuple!(A B C D E F G H I);
impl_alt_tuple!(A B C D E F G H I J);
impl_alt_tuple!(A B C D E F G H I J K);
impl_alt_tuple!(A B C D E F G H I J K L);
impl_alt_tuple!(A B C D E F G H I J K L M);
impl_alt_tuple!(A B C D E F G H I J K L M N);
impl_alt_tuple!(A B C D E F G H I J K L M N O);

