#![deny(unused_imports)]
use thiserror::Error;
use super::Operation;


#[derive(Debug, Error,Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PostfixerErrorKind {
    #[error("Expected an operator, got {0}")]
    ExpectedOperator(String),
    #[error("Expected an odd number of args, got {0}")]
    NeedOddAmountOfArgs(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Error)]
pub enum OperationErrorKind {
    #[error("Divide by zero")]
    DivideByZero,
    #[error("Overflow")]
    Overflow,
    #[error("Illegal bit operation")]
    IllegalBitOp,
    #[error("Illegal shift operation")]
    IllegalShift,
    #[error("Incompatible operands")]
    IncompatibleOperands,
    #[error("Illegal negation")]
    IllegalNegation,
}

#[derive(Clone, Debug, PartialEq, Error)]
pub enum GenericEvalErrorKind {
    #[error("Expected an operator, got {0}")]
    UnexpectedOp(Operation),
    #[error("Unable to evaluate expression")]
    UnableToEvaluate,
    #[error("Did not expecte a value")]
    UnexpectedValue,
    #[error("Did not evaluate all items")]
    UnevaluatedTerms,
    #[error("Stack unexpectedly empty")]
    StackEmpty,
    #[error("Expected a value")]
    ExpectedValue,
    #[error("Expected an operator")]
    ExpectedOperator,
    #[error(transparent)]
    OperatorError(#[from] OperationErrorKind),
    #[error(transparent)]
    PostFixError(#[from] PostfixerErrorKind),
}

pub type GenericEvalResult<T> = Result<T, GenericEvalErrorKind>;

impl<T> From<OperationError<T>> for GenericEvalErrorKind {
    fn from(_: OperationError<T>) -> Self {
        todo!()
    }
}

pub type OperationError<T> = Result<T, OperationErrorKind>;


