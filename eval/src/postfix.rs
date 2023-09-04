use super::PostfixerErrorKind;
use crate::Stack;

pub trait GetPriority {
    fn priority(&self) -> Option<usize> {
        None
    }
    fn is_op(&self) -> bool {
        self.priority().is_some()
    }
}

#[derive(Debug, Clone)]
pub struct PostFixer<I: Clone + GetPriority> {
    opstack: Stack<I>,
    ret: Vec<I>,
}

impl<I: Clone + GetPriority + std::fmt::Debug> Default for PostFixer<I> {
    fn default() -> Self {
        Self {
            opstack: Stack::new(),
            ret: vec![],
        }
    }
}

impl<I: Clone + GetPriority + std::fmt::Debug> PostFixer<I> {
    pub fn new() -> Self {
        Self::default()
    }

    fn emit(&mut self, i: &I) {
        self.ret.push(i.clone())
    }

    fn push(&mut self, op: &I) -> Result<(), PostfixerErrorKind> {
        if op.is_op() {
            self.opstack.push(op.clone());
            Ok(())
        } else {
            Err(PostfixerErrorKind::ExpectedOperator(format!("{op:?}")))
        }
    }

    fn flush(&mut self) {
        let mut v = self.opstack.flush();
        self.ret.append(&mut v);
    }

    fn finalize(&mut self) -> Vec<I> {
        self.flush();
        std::mem::take(&mut self.ret)
    }

    fn top_pri(&self) -> usize {
        if let Some(top_op) = self.opstack.front() {
            top_op.priority().unwrap()
        } else {
            0
        }
    }

    pub fn get_postfix(&mut self, ops: &[I]) -> Result<Vec<I>, PostfixerErrorKind> {
        let len = ops.len();

        if len % 2 == 0 && len != 0 {
            return Err(PostfixerErrorKind::NeedOddAmountOfArgs(len));
        }

        match len {
            0 => Ok(vec![]),
            1 => Ok(ops.to_vec()),
            _ => {
                let mut it = ops.iter();
                let lhs = it.next().unwrap();

                let mut next_pair = || it.next().and_then(|op| it.next().map(|rhs| (op, rhs)));

                let (op, rhs) = next_pair().unwrap();
                self.emit(lhs);
                self.emit(rhs);
                self.push(op)?;

                while let Some((op, rhs)) = next_pair() {
                    let top_pri = self.top_pri();

                    let this_pri = op
                        .priority()
                        .ok_or_else(|| PostfixerErrorKind::ExpectedOperator(format!("{op:?}")))?;

                    if top_pri >= this_pri {
                        self.flush();
                    }

                    self.push(op)?;
                    self.emit(rhs);
                }

                Ok(self.finalize())
            }
        }
    }
}

pub fn to_postfix<I>(i: &[I]) -> Result<Vec<I>, PostfixerErrorKind>
where
    I: Clone + GetPriority + std::fmt::Debug,
{
    let mut x = PostFixer::new();
    x.get_postfix(i)
}

#[allow(unused_imports)]
mod test {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};
    impl GetPriority for isize {}

    fn to_string(vs: &[char]) -> String {
        String::from_iter(vs.iter())
    }

    impl GetPriority for char {
        fn priority(&self) -> Option<usize> {
            match self {
                '/' => Some(5),
                '*' => Some(4),
                '+' => Some(2),
                '-' => Some(2),
                '>' => Some(1),
                '<' => Some(1),
                _ => None,
            }
        }
    }

    fn to_args(test: &str) -> (char, Vec<(char, char)>) {
        let mut it = test.chars();

        let first = it.next().unwrap();
        let mut rest = vec![];

        while let Some(op) = it.next() {
            let arg = it.next().unwrap();
            rest.push((op, arg))
        }
        (first, rest)
    }

    pub fn eval(e: &[char]) -> i64 {
        let mut s: Stack<i64> = Stack::new();

        let to_i64 = |c: char| (c as i64) - '0' as i64;

        for i in e.iter() {
            if i.is_op() {
                let (rhs, lhs) = s.pop_pair().expect("Can't pop pair!");
                let res = match i {
                    '*' => lhs * rhs,
                    '<' => lhs << rhs,
                    '>' => lhs >> rhs,
                    '/' => lhs / rhs,
                    '+' => lhs + rhs,
                    '-' => lhs - rhs,
                    _ => panic!(),
                };
                s.push(res);
            } else {
                s.push(to_i64(*i));
            }
        }

        s.pop().expect("Can't pop!")
    }

    #[test]
    fn postix() {
        let tests = vec![
            ("1+2*4", 1 + 2 * 4),
            ("1/2*4", 1 / 2 * 4),
            ("8-1+3", 8 - 1 + 3),
            ("8-1<9", 8 - 1 << 9),
            ("9+8>1", 9 + 8 >> 1),
        ];

        for (test, res) in tests {
            println!("testing : {test}");
            let mut x = PostFixer::new();
            let args: Vec<_> = test.chars().collect();
            let ret = x.get_postfix(&args).unwrap();
            let result = eval(&ret);
            let ret_str = to_string(&ret);
            println!("Infix {}", ret_str);
            assert_eq!(result, res);
        }

        let test = "8-1+3";
        let desired = "81-3+";

        println!("Input: {}", test);
        let mut x = PostFixer::new();

        let args: Vec<_> = test.chars().collect();
        let ret = x.get_postfix(&args).unwrap();
        let result = eval(&ret);
        let ret_str = to_string(&ret);

        println!("{:?}", test);
        println!("{:?}", ret_str);
        println!("{:?}", ret);
        assert_eq!(ret_str, desired);
        assert_eq!(result, 10);
    }
}
