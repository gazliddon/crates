# grl-eval

Small, generic expression-evaluation helpers used by Gazm.

The crate converts a flat infix expression (`value, operator, value, ...`)
to postfix notation and evaluates the resulting stream using caller-provided
value and operator implementations.

The API is intentionally narrow: grouping and unary operators must be
handled by the caller before conversion.
