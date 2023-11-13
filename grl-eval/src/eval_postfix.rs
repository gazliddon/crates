#![deny(unused_imports)]
use std::collections::VecDeque;

use super::{GenericEvalErrorKind, GetPriority, OperationError, pop_pair};


/// Traits a value in an expression must support
pub trait OperatorTraits:
    std::ops::Add<Output = OperationError<Self>>
    + std::ops::Sub<Output = OperationError<Self>>
    + std::ops::Mul<Output = OperationError<Self>>
    + std::ops::Div<Output = OperationError<Self>>
    + std::ops::Rem<Output = OperationError<Self>>
    + std::ops::BitOr<Output = OperationError<Self>>
    + std::ops::BitAnd<Output = OperationError<Self>>
    + std::ops::BitXor<Output = OperationError<Self>>
    + std::ops::Shl<Output = OperationError<Self>>
    + std::ops::Shr<Output = OperationError<Self>>
    + Sized
    + Clone
{
}

/// Classification of what kind of item this is
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ExprItemKind {
    Expression,
    Value,
    Operator,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Operation {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    BitOr,
    BitAnd,
    BitXor,
    ShiftRight,
    ShiftLeft,
}

impl GetPriority for Operation {
    fn priority(&self) -> Option<usize> {
        use Operation::*;
        let ret = match self {
            Div => 12,
            Rem => 12,
            Mul => 12,
            Add => 11,
            Sub => 11,
            ShiftRight => 10,
            ShiftLeft => 10,
            BitAnd => 9,
            BitXor => 8,
            BitOr => 7,
        };
        Some(ret)
    }
}

impl std::fmt::Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// Traits needed for classification of an item
/// is it a Value, an operator or an expression
pub trait ItemTraits: Clone {
    type ExprValue: OperatorTraits;

    fn item_type(&self) -> ExprItemKind {
        use ExprItemKind::*;

        if self.is_value() {
            Value
        } else if self.is_expression() {
            Expression
        } else if self.is_op() {
            Operator
        } else {
            panic!()
        }
    }

    fn value(&self) -> Option<Self::ExprValue>;
    fn op(&self) -> Option<Operation>;
    fn expr(&self) -> Option<&Vec<Self>>;

    fn is_expression(&self) -> bool {
        self.expr().is_some()
    }

    fn is_op(&self) -> bool {
        self.op().is_some()
    }

    fn is_value(&self) -> bool {
        self.value().is_some()
    }
}

/// Trait needed to evaluate an item
/// Evals from an item to a value
pub trait Eval<I, ERR>
where
    ERR: From<GenericEvalErrorKind>,
    I: ItemTraits,
{
    fn eval_expr(&self, i: &I) -> Result<I::ExprValue, ERR>;
}

////////////////////////////////////////////////////////////////////////////////
#[derive(Clone)]
enum StackItem<'a, I>
where
    I: ItemTraits,
{
    Value(<I as ItemTraits>::ExprValue),
    Item(&'a I),
}

impl<'a, I> StackItem<'a, I>
where
    I: ItemTraits,
{
    pub fn item(&self) -> Option<&I> {
        match self {
            StackItem::Item(i) => Some(i),
            _ => None,
        }
    }
}

impl<'a, I> From<&'a I> for StackItem<'a, I>
where
    I: ItemTraits,
{
    fn from(item: &'a I) -> Self {
        use ExprItemKind::*;
        match item.item_type() {
            Value => StackItem::Value(item.value().unwrap()),
            _ => StackItem::Item(item),
        }
    }
}

impl<'a, I> ItemTraits for StackItem<'a, I>
where
    I: ItemTraits,
{
    type ExprValue = I::ExprValue;

    fn value(&self) -> Option<Self::ExprValue> {
        match self {
            StackItem::Value(v) => Some(v.clone()),
            _ => None,
        }
    }

    fn op(&self) -> Option<Operation> {
        match self {
            StackItem::Item(i) => i.op(),
            _ => None,
        }
    }

    fn expr(&self) -> Option<&Vec<Self>> {
        None
    }
}

pub fn evaluate_postfix_expr_2<'a, I, E, ERR>(
    items: impl Iterator<Item = &'a I>,
    _evaluator: &E,
) -> Result<I::ExprValue, ERR>
where
    I: ItemTraits + 'a,
    E: Eval<I, ERR>,
    ERR: From<GenericEvalErrorKind>,
{
    use GenericEvalErrorKind::*;
    use Operation::*;

    let mut s: VecDeque<StackItem<'a, I>> = VecDeque::new();

    let to_err = |_e| panic!("{_e:#?}");

    for i in items.map(StackItem::from) {

        let ret = match i.item_type() {
            ExprItemKind::Expression => StackItem::Value(_evaluator.eval_expr(i.item().unwrap())?),

            ExprItemKind::Operator => {
                let op = i.op().ok_or_else(|| to_err(ExpectedOperator))?;

                let (rhs, lhs) = pop_pair(&mut s).expect("Can't pop pair?");
                let lhs = lhs.value().ok_or_else(|| to_err(ExpectedValue))?;
                let rhs = rhs.value().ok_or_else(|| to_err(ExpectedValue))?;

                match op {
                    Mul => lhs * rhs,
                    Div => lhs / rhs,
                    Add => lhs + rhs,
                    Sub => lhs - rhs,
                    BitAnd => lhs & rhs,
                    BitXor => lhs ^ rhs,
                    BitOr => lhs | rhs,
                    ShiftLeft => lhs << rhs,
                    ShiftRight => lhs >> rhs,
                    Rem => lhs % rhs,
                }
                .map(|i| StackItem::Value(i))
                .map_err(|e| to_err(GenericEvalErrorKind::from(e)))?
            }

            ExprItemKind::Value => i,
        };

        s.push_front(ret)
    }
    match s.len() {
        // Nothing on top of stack
        0 => Err(StackEmpty),
        // Something, try and extract the value
        1 => s
            .pop_front()
            .expect("Can't pop!")
            .value()
            .ok_or(ExpectedValue),
        // Too many things on stack
        _ => Err(UnevaluatedTerms),
    }
    .map_err(|e| e.into())
}


pub fn evaluate_postfix_expr<I, E, ERR>(
    items: impl Iterator<Item = I>,
    evaluator: &E,
) -> Result<I::ExprValue, (usize, ERR)>
where
    I: ItemTraits + From<I::ExprValue>,
    E: Eval<I, ERR>,
    ERR: From<GenericEvalErrorKind>,
{
    use GenericEvalErrorKind::*;
    use Operation::*;

    // todo
    // check that we have enough items in the iterator?

    let mut s: VecDeque<I> = VecDeque::new();
    let idx_err = |idx, e| -> (usize, ERR) { (idx, ERR::from(e)) };

    for (idx, i) in items.enumerate() {
        let to_err = |e| -> (usize, ERR) { idx_err(idx, e) };

        let i = match i.item_type() {
            ExprItemKind::Expression => evaluator.eval_expr(&i).map_err(|e| (idx, e))?.into(),

            ExprItemKind::Operator => {
                let op = i.op().ok_or(to_err(ExpectedOperator))?;

                let (rhs, lhs) = pop_pair(&mut s).expect("Can't pop pair!");
                let lhs = lhs.value().ok_or(to_err(ExpectedValue))?;
                let rhs = rhs.value().ok_or(to_err(ExpectedValue))?;

                match op {
                    Mul => lhs * rhs,
                    Div => lhs / rhs,
                    Add => lhs + rhs,
                    Sub => lhs - rhs,
                    BitAnd => lhs & rhs,
                    BitXor => lhs ^ rhs,
                    BitOr => lhs | rhs,
                    ShiftLeft => lhs << rhs,
                    ShiftRight => lhs >> rhs,
                    Rem => lhs % rhs,
                }
                .map(|v| I::from(v))
                .map_err(|e| to_err(GenericEvalErrorKind::from(e)))?
            }

            ExprItemKind::Value => i.clone(),
        };

        s.push_front(i)
    }

    match s.len() {
        // Nothing on top of stack
        0 => Err(StackEmpty),
        // Something, try and extract the value
        1 => s.pop_front().expect("Can't pop!").value().ok_or(ExpectedValue),
        // Too many things on stack
        _ => Err(UnevaluatedTerms),
    }
    .map_err(|e| idx_err(0, e))
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExprItem<V: GetPriority> {
    Op(Operation),
    Val(V),
    Expr(Vec<ExprItem<V>>),
}

impl<V: GetPriority> ExprItem<V> {
    pub fn from_val(v: V) -> Self {
        ExprItem::Val(v)
    }

    pub fn from_op(op: Operation) -> Self {
        ExprItem::Op(op)
    }
}

impl<V: GetPriority> GetPriority for ExprItem<V> {
    fn priority(&self) -> Option<usize> {
        use ExprItem::*;
        match self {
            Op(e) => e.priority(),
            Val(_) => None,
            Expr(_) => None,
        }
    }
}

impl<V: GetPriority + OperatorTraits + Clone> ItemTraits for ExprItem<V> {
    type ExprValue = V;
    fn value(&self) -> Option<V> {
        match self {
            ExprItem::Val(v) => Some(v.clone()),
            _ => None,
        }
    }

    fn op(&self) -> Option<Operation> {
        match self {
            ExprItem::Op(v) => Some(*v),
            _ => None,
        }
    }

    fn expr(&self) -> Option<&Vec<ExprItem<V>>> {
        match self {
            ExprItem::Expr(v) => Some(v),
            _ => None,
        }
    }
}

impl<V: GetPriority> From<V> for ExprItem<V> {
    fn from(v: V) -> Self {
        ExprItem::Val(v)
    }
}

mod test {

    // type OpValLocal = ExprItem<isize>;

    // struct Evaluator {}

    // impl EvalExpr<isize, OpValLocal, GenericEvalError> for Evaluator {
    //     fn eval_expr(&self, _i: &OpValLocal) -> Result<OpValLocal, GenericEvalError> {
    //         todo!()
    //     }
    // }

    #[test]
    fn test_eval() {
        // use Operation::*;
        // use ExprItem::*;

        // let infix_items: Vec<OpValLocal> =
        //     vec![Val(10), Op(Add), Val(20), Op(Div), Val(5)];

        // let mut pfix = PostFixer::new();
        // let items = pfix.get_postfix(infix_items).unwrap();

        // // let items: Vec<OpVal> = vec![10.into(), 10.into(), Add.into(), 5.into(), Div.into()];
        // let evaluator = Evaluator {};
        // let x = generic_postfix_eval(items.into_iter(), &evaluator);
        // let desired = Ok(Val(14));
        // assert_eq!(x, desired);
    }
}
