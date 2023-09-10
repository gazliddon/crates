#![allow(unused)]

use unraveler::{Item, ParseError,ParseErrorKind, tag,pair, many0, many1, alt, tuple, any, Severity};

type Span<'a> = unraveler::Span<'a, Token>;

#[derive(Debug,Clone,Copy)]
struct NewError {}

impl ParseError<Span<'_>> for NewError {
    fn from_error_kind(input: Span, kind: ParseErrorKind, sev: Severity) -> Self {
        println!("AN ERROR: {:?}", kind);
        NewError {  }
    }

    fn append(input: Span, kind: ParseErrorKind, other: Self) -> Self {
        todo!()
    }

    fn change_kind(self, kind: ParseErrorKind) -> Self {
        todo!()
    }

    fn set_severity(self, sev: Severity)-> Self {
        todo!()
    }

    fn severity(&self) -> Severity {
        Severity::Error
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum TokenKind {
    A,
    B,
    C,
}

impl Item for TokenKind {
    type Kind = TokenKind;

    fn get_kind(&self) -> TokenKind {
        *self
    }

}

#[derive(Copy,Clone, Debug)]
struct Token {
    kind: TokenKind,
    x : usize,
}

impl Token {
    pub fn new(kind : TokenKind) -> Self {
        Self {kind, x: 255}
    }
}

impl Item for Token {
    type Kind = TokenKind;

    fn get_kind(&self) -> TokenKind {
        self.kind
    }
}

fn to_kinds(sp : Span) -> Vec<TokenKind> {
    sp.kinds_iter().collect()
}

fn to_tokens(kinds : &[TokenKind]) -> Vec<Token> {
    kinds.iter().cloned().map(Token::new).collect()
}

#[test]
fn test_norm() {
    use TokenKind::*;
    let doc = to_tokens(&[A,B,B,A,C,A,B]);
    let input = Span::from_slice(&doc);

    let (rest,(left,right)) = test_fn(input).unwrap();

    println!("0: {:?} {:?}", to_kinds(input), input.get_range());
    println!("1: {:?} {:?}", to_kinds(left), left.get_range());
    println!("2: {:?} {:?}", to_kinds(right), right.get_range());
    println!("r: {:?} {:?}", to_kinds(rest), rest.get_range());
    // assert!(false)
}


#[test]
fn test_tuple() -> Result<(),NewError> {
    use TokenKind::*;
    let doc = to_tokens(&[A,B,B,A,A,A,B]);
    let input = Span::from_slice(&doc);

    let (rest,(a,b,c)) = tuple((tag([A,B]),tag([B]),many0(tag([A]))))(input)?;
    assert_eq!(to_kinds(a),[A,B]);
    assert_eq!(to_kinds(b),[B]);
    let c : Vec<_> = c.into_iter().map(|x| to_kinds(x)).flatten().collect();
    println!("VEC: {:?}",c);
    assert!(false);

    Ok(())
}

#[test]
fn test_alt() -> Result<(),NewError>{
    use TokenKind::*;
    let doc = to_tokens(&[B,A,A,A,A,A,B]);
    let input = Span::from_slice(&doc);

    let (rest, matched) = alt(
        (tag([B,A]), tag([A,A]))
    )(input)?;

    println!("ret: {:?}", to_kinds(matched));
    // assert!(false);

    Ok(())
}

#[test]
fn test_many() -> Result<(),NewError>{
    use TokenKind::*;

    let doc = to_tokens(&[B,A,A,A,A,A,B]);

    let input = Span::from_slice(&doc);

    let res = pair(tag([B]), many0(tag([A])))(input);

    let (rest,(open,v)) = res?;

    let v : Vec<_> = v.iter().map(|i| to_kinds(*i)).flatten().collect();


    println!("o: {:?}", to_kinds(open));
    println!("v: {:?}", v);
    println!("r: {:?} {:?}", to_kinds(rest), rest.get_range());


    assert_eq!(to_kinds(open), [B]);
    assert_eq!(v, [A,A,A,A,A]);
    assert_eq!(to_kinds(rest),[B]);

    Ok(())

}

fn test_fn(input : Span) -> Result<(Span,( Span, Span )),NewError> {
    use TokenKind::*;

    let (rest,( matched,x)) = pair(
        tag(&[A,B]),tag(&[B,A])
    )(input)?;

    Ok((rest,( matched,x)))
}


