// #![allow(unused)]
// use unraveler::{ tag, ParseErrorKind,ParseError };

// type Span<'a> = unraveler::Span<'a,u8>;

// #[derive(Debug,Clone,Copy)]
// struct NewError {}

// impl ParseError<Span<'_>> for NewError {
//     fn from_error_kind(input: &Span, kind: ParseErrorKind) -> Self {
//         todo!()
//     }

//     fn append(input: &Span, kind: ParseErrorKind, other: Self) -> Self {
//         todo!()
//     }
// }

// // #[test]
// fn tester() {
//     let doc = Span::from_slice(b"Hello! how are you?");
//     let (rest,matched) = test_fn(doc).unwrap();
//     assert!(false)
// }

// fn test_fn(input : Span) -> Result<(Span,Span),NewError> {
//     let x = Span::from_slice(b"Hello!");
//     let (rest,matched) = tag(x)(input)?;
//     println!("{matched:?}");
//     Ok((rest,matched))
// }


