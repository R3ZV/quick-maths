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

Various properties are being teste to make sure our language is consistent with
the expected behavior, here is a comprehensive list of properties being tested for:
Here is a complete, categorized list of all the mathematical and logical properties currently being tested for each operator across your interpreter tests:

### Arithmetic Operators (+, -, *, /)

#### Addition (+)

- Commutative: a + b == b + a
- Associative: (a + b) + c == a + (b + c)
- Identity Element: a + 0 == a (and 0 + a == a)

#### Multiplication (*)

- Commutative: a * b == b * a
- Associative: (a * b) * c == a * (b * c)

#### Mixed Arithmetic (* and +)

- Distributive Property: a * (b + c) == (a * b) + (a * c)
- Subtraction (-)
- Self-Annihilation (Nilpotence): a - a == 0
- Division (/)
- Self-Division: a / a == 1 (where a != 0)

### Comparison Operators (<, <=, >, >=, ==, !=)

#### Less Than / Greater Than (<, >)

- Irreflexive: a < a is false, and a > a is false.
- Mirror Symmetry: a > b == b < a

#### Less Eq / Greater Eq (<=, >=)
- Reflexive: a <= a is true, and a >= a is true.
- Mirror Symmetry: a <= b == b >= a
- Transitive: If a <= b and b <= c, then a <= c.

#### Not Equal (!=)
- Irreflexive: a != a is false.
- Symmetric: a != b == b != a

### Logical Operators (&, |, !)

#### AND (&)

- Commutative: a & b == b & a
- Identity: a & true == a
- Idempotent: a & a == a
- Domination (Annihilation): a & false == false
- Complement (Contradiction): a & !a == false

#### OR (|)

- Commutative: a | b == b | a
- Identity: a | false == a
- Idempotent: a | a == a
- Domination (Annihilation): a | true == true
- Complement (Excluded Middle): a | !a == true

#### NOT (!)

- Base Cases: !true == false and !false == true
- Double Negation (Involution): !!a == a

#### Mixed Logical (&, |, !)

- De Morgan's Law (AND): !(a & b) == !a | !b
- De Morgan's Law (OR): !(a | b) == !a & !b

Besides the testing of the operators we also check a misc of properties:
- Determinism of the system
- A round trip from AST to parsable string and back to AST
- Ensure that the system doesn't panic on arbitrary input
- Ensure that it catches type mismatches
