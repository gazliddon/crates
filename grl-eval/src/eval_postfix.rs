#![deny(unused_imports)]
use std::collections::VecDeque;

use super::{pop_pair, GenericEvalErrorKind, GetPriority, OperationError};

/// Operations a value in an expression must support.
///
/// Arithmetic is fallible because callers may need to report overflow,
/// division by zero, or an otherwise invalid operation for their value type.
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

/// Classification of an expression item.
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

/// Traits needed to classify and inspect an expression item.
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
            // An implementation returning `None` for all three accessors is
            // invalid.  Keep this as a programmer-error panic rather than
            // silently treating malformed items as values.
            panic!("expression item has no value, operator, or expression")
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

/// Evaluates a nested expression item into a value.
pub trait Eval<I, ERR>
where
    ERR: From<GenericEvalErrorKind>,
    I: ItemTraits,
{
    fn eval_expr(&self, i: &I) -> Result<I::ExprValue, ERR>;
}

////////////////////////////////////////////////////////////////////////////////
/// Evaluate postfix items while retaining the source item for expression
/// callbacks.  The value stack deliberately contains only evaluated values;
/// this lets callers provide borrowed AST nodes without requiring an
/// `I: From<I::ExprValue>` implementation.
fn evaluate_postfix_refs<'a, I, E, ERR>(
    items: impl Iterator<Item = &'a I>,
    evaluator: &E,
) -> Result<I::ExprValue, (usize, ERR)>
where
    I: ItemTraits + 'a,
    E: Eval<I, ERR>,
    ERR: From<GenericEvalErrorKind>,
{
    use GenericEvalErrorKind::*;
    use Operation::*;

    let mut values: VecDeque<I::ExprValue> = VecDeque::new();
    let mut last_idx = 0;

    for (idx, item) in items.enumerate() {
        last_idx = idx;
        let error = |kind| (idx, ERR::from(kind));
        let value = match item.item_type() {
            ExprItemKind::Expression => evaluator.eval_expr(item).map_err(|e| (idx, e))?,
            ExprItemKind::Value => item.value().ok_or_else(|| error(ExpectedValue))?,
            ExprItemKind::Operator => {
                let op = item.op().ok_or_else(|| error(ExpectedOperator))?;
                let (rhs, lhs) = pop_pair(&mut values).ok_or_else(|| error(StackEmpty))?;

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
                .map_err(|e| error(GenericEvalErrorKind::from(e)))?
            }
        };
        values.push_front(value);
    }

    let result = match values.len() {
        0 => Err(StackEmpty),
        1 => values.pop_front().ok_or(StackEmpty),
        _ => Err(UnevaluatedTerms),
    };
    result.map_err(|kind| (last_idx, ERR::from(kind)))
}

pub fn evaluate_postfix_expr<I, E, ERR>(
    items: impl Iterator<Item = I>,
    evaluator: &E,
) -> Result<I::ExprValue, (usize, ERR)>
where
    I: ItemTraits,
    E: Eval<I, ERR>,
    ERR: From<GenericEvalErrorKind>,
{
    let owned: Vec<I> = items.collect();
    evaluate_postfix_refs(owned.iter(), evaluator)
}

/// A convenient generic expression-item implementation for clients that do
/// not need a custom AST node type.
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

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct TestValue(i64);

    macro_rules! arithmetic {
        ($trait:ident, $method:ident, $operation:expr) => {
            impl std::ops::$trait for TestValue {
                type Output = OperationError<Self>;

                fn $method(self, rhs: Self) -> Self::Output {
                    Ok(TestValue($operation(self.0, rhs.0)))
                }
            }
        };
    }

    arithmetic!(Add, add, |lhs, rhs| lhs + rhs);
    arithmetic!(Sub, sub, |lhs, rhs| lhs - rhs);
    arithmetic!(Mul, mul, |lhs, rhs| lhs * rhs);
    arithmetic!(Div, div, |lhs, rhs| lhs / rhs);
    arithmetic!(Rem, rem, |lhs, rhs| lhs % rhs);
    arithmetic!(BitOr, bitor, |lhs, rhs| lhs | rhs);
    arithmetic!(BitAnd, bitand, |lhs, rhs| lhs & rhs);
    arithmetic!(BitXor, bitxor, |lhs, rhs| lhs ^ rhs);
    arithmetic!(Shl, shl, |lhs, rhs| lhs << rhs);
    arithmetic!(Shr, shr, |lhs, rhs| lhs >> rhs);

    impl OperatorTraits for TestValue {}
    impl GetPriority for TestValue {}

    struct Evaluator;

    impl Eval<ExprItem<TestValue>, GenericEvalErrorKind> for Evaluator {
        fn eval_expr(
            &self,
            _item: &ExprItem<TestValue>,
        ) -> Result<TestValue, GenericEvalErrorKind> {
            Ok(TestValue(0))
        }
    }

    #[test]
    fn evaluates_postfix_expression() {
        use ExprItem::{Op, Val};

        let items = vec![
            Val(TestValue(10)),
            Val(TestValue(20)),
            Op(Operation::Add),
            Val(TestValue(5)),
            Op(Operation::Div),
        ];

        assert_eq!(
            evaluate_postfix_expr(items.into_iter(), &Evaluator),
            Ok(TestValue(6))
        );
    }

    #[test]
    fn reports_stack_underflow_instead_of_panicking() {
        let items = vec![ExprItem::<TestValue>::from_op(Operation::Add)];
        assert!(matches!(
            evaluate_postfix_expr(items.into_iter(), &Evaluator),
            Err((0, GenericEvalErrorKind::StackEmpty))
        ));
    }
}
