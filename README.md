# Quick maths

## REPL

If you want to test the language you can run our interactive REPL by running
the project `cargo run`.

## Specification

Quick maths is an arithemtical language, it supports following unary operations:
- not (!a)
- minus (-a)

As well as the following binary operatos:
- Addition (a + b)
- Subtraction (a - b)
- Multiplication (a * b)
- Division (a / b)
- Less than (a < b)
- Less than or equal (a <= b)
- Greater than (a > b)
- Greater than or equal (a >= b)
- Equal (a == b)
- Not equal (a != b)
- And logical (a & b)
- Or logical (a | b)


## Testing via quick check

You can run the tests via `cargo test` from the root directory.

The `quickcheck` crate runs by default `100` tests for each properity you declare
with `#[quickcheck]` macro, if you want to increase the number of tests, you
can do: `QUICKCHECK_TESTS=x cargo test` where `x` is how many tests you want to run.
