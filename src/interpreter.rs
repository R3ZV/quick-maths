use std::collections::HashMap;

use crate::common::Value;
use crate::error::Error;
use crate::lexer::{Lexer, print_tokens};
use crate::parser::Parser;

pub struct Interpreter {
    pgm_state: HashMap<String, Value>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            pgm_state: HashMap::new(),
        }
    }

    pub fn run(&mut self, instr: &str) -> Result<Value, Error> {
        let mut lexer = Lexer::new(instr);
        let tokens = lexer.tokenize()?;
        print_tokens(&tokens);

        let mut parser: Parser = Parser::new(tokens);
        let expr = parser.parse_expr()?;

        expr.eval(&mut self.pgm_state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quickcheck::TestResult;
    use quickcheck_macros::quickcheck;

    fn is_parseable(val: i32) -> bool {
        val > i32::MIN
    }

    #[quickcheck]
    fn prop_tokenizer_never_panics(input: String) -> bool {
        let mut lexer = Lexer::new(input.as_str());
        let _ = lexer.tokenize();
        true
    }

    #[quickcheck]
    fn prop_interpreter_never_panics_on_garbage(input: String) -> bool {
        let mut interpreter = Interpreter::new();
        let _ = interpreter.run(&input);
        true
    }

    #[quickcheck]
    fn prop_addition_is_commutative(a: i32, b: i32) -> TestResult {
        if a.checked_add(b).is_none() || !is_parseable(a) || !is_parseable(b) {
            return TestResult::discard();
        }

        let mut interp = Interpreter::new();

        let expr1 = format!("{} + {}\n", a, b);
        let expr2 = format!("{} + {}\n", b, a);

        eprintln!("Evaluating: {}", expr1);
        let res1 = interp.run(&expr1).unwrap();

        eprintln!("Evaluating: {}", expr2);
        let res2 = interp.run(&expr2).unwrap();

        TestResult::from_bool(res1 == res2)
    }

    #[quickcheck]
    fn prop_addition_is_associative(a: i32, b: i32, c: i32) -> TestResult {
        if !is_parseable(a) || !is_parseable(b) || !is_parseable(c) {
            return TestResult::discard();
        }

        // Ensure no additions overflow
        let ab = a.checked_add(b);
        if ab.is_none() || !is_parseable(ab.unwrap()) {
            return TestResult::discard();
        }
        if ab.unwrap().checked_add(c).is_none() {
            return TestResult::discard();
        }

        let bc = b.checked_add(c);
        if bc.is_none() || !is_parseable(bc.unwrap()) {
            return TestResult::discard();
        }
        if a.checked_add(bc.unwrap()).is_none() {
            return TestResult::discard();
        }

        let mut interp = Interpreter::new();

        let expr1 = format!("({} + {}) + {}\n", a, b, c);
        let expr2 = format!("{} + ({} + {})\n", a, b, c);

        let res1 = interp.run(&expr1).unwrap();
        let res2 = interp.run(&expr2).unwrap();

        TestResult::from_bool(res1 == res2)
    }

    #[quickcheck]
    fn prop_addition_identity(a: i32) -> TestResult {
        if !is_parseable(a) {
            return TestResult::discard();
        }

        let mut interp = Interpreter::new();
        let expr1 = format!("{} + 0\n", a);
        let expr2 = format!("0 + {}\n", a);

        let res1 = interp.run(&expr1).unwrap();
        let res2 = interp.run(&expr2).unwrap();

        TestResult::from_bool(res1 == Value::Int(a) && res2 == Value::Int(a))
    }

    #[quickcheck]
    fn prop_multiplication_is_commutative(a: i32, b: i32) -> TestResult {
        if !is_parseable(a) || !is_parseable(b) {
            return TestResult::discard();
        }
        if a.checked_mul(b).is_none() {
            return TestResult::discard();
        }

        let mut interp = Interpreter::new();

        let expr1 = format!("{} * {}\n", a, b);
        let expr2 = format!("{} * {}\n", b, a);

        let res1 = interp.run(&expr1).unwrap();
        let res2 = interp.run(&expr2).unwrap();

        TestResult::from_bool(res1 == res2)
    }

    #[quickcheck]
    fn prop_multiplication_is_associative(a: i32, b: i32, c: i32) -> TestResult {
        if !is_parseable(a) || !is_parseable(b) || !is_parseable(c) {
            return TestResult::discard();
        }

        // no multi overflow
        let ab = a.checked_mul(b);
        if ab.is_none() || !is_parseable(ab.unwrap()) {
            return TestResult::discard();
        }
        if ab.unwrap().checked_mul(c).is_none() {
            return TestResult::discard();
        }

        let bc = b.checked_mul(c);
        if bc.is_none() || !is_parseable(bc.unwrap()) {
            return TestResult::discard();
        }
        if a.checked_mul(bc.unwrap()).is_none() {
            return TestResult::discard();
        }

        let mut interp = Interpreter::new();

        let expr1 = format!("({} * {}) * {}\n", a, b, c);
        let expr2 = format!("{} * ({} * {})\n", a, b, c);

        let res1 = interp.run(&expr1).unwrap();
        let res2 = interp.run(&expr2).unwrap();

        TestResult::from_bool(res1 == res2)
    }

    #[quickcheck]
    fn prop_distributive_property(a: i32, b: i32, c: i32) -> TestResult {
        if !is_parseable(a) || !is_parseable(b) || !is_parseable(c) {
            return TestResult::discard();
        }

        // Check a * (b + c)
        let bc = b.checked_add(c);
        if bc.is_none() || !is_parseable(bc.unwrap()) {
            return TestResult::discard();
        }
        if a.checked_mul(bc.unwrap()).is_none() {
            return TestResult::discard();
        }

        // Check a*b + a*c
        let ab = a.checked_mul(b);
        let ac = a.checked_mul(c);
        if ab.is_none() || ac.is_none() || !is_parseable(ab.unwrap()) || !is_parseable(ac.unwrap())
        {
            return TestResult::discard();
        }
        if ab.unwrap().checked_add(ac.unwrap()).is_none() {
            return TestResult::discard();
        }

        let mut interp = Interpreter::new();

        let expr1 = format!("{} * ({} + {})\n", a, b, c);
        let expr2 = format!("{} * {} + {} * {}\n", a, b, a, c);

        let res1 = interp.run(&expr1).unwrap();
        let res2 = interp.run(&expr2).unwrap();

        TestResult::from_bool(res1 == res2)
    }

    #[quickcheck]
    fn prop_subtraction_self_is_zero(a: i32) -> TestResult {
        if !is_parseable(a) {
            return TestResult::discard();
        }

        let mut interp = Interpreter::new();
        let expr = format!("{} - {}\n", a, a);
        let res = interp.run(&expr).unwrap();

        TestResult::from_bool(res == Value::Int(0))
    }

    #[quickcheck]
    fn prop_division_by_self_is_one(a: i32) -> TestResult {
        // Discard 0 to avoid Division By Zero panics
        if a == 0 || !is_parseable(a) {
            return TestResult::discard();
        }

        let mut interp = Interpreter::new();
        let expr = format!("{} / {}\n", a, a);
        let res = interp.run(&expr).unwrap();

        TestResult::from_bool(res == Value::Int(1))
    }
    #[quickcheck]
    fn prop_less_than_or_equal_reflexive(a: i32) -> TestResult {
        if !is_parseable(a) {
            return TestResult::discard();
        }

        let mut interp = Interpreter::new();
        let expr = format!("{} <= {}\n", a, a);
        let res = interp.run(&expr).unwrap();

        TestResult::from_bool(res == Value::Bool(true))
    }

    #[quickcheck]
    fn prop_greater_than_or_equal_reflexive(a: i32) -> TestResult {
        if !is_parseable(a) {
            return TestResult::discard();
        }

        let mut interp = Interpreter::new();
        let expr = format!("{} >= {}\n", a, a);
        let res = interp.run(&expr).unwrap();

        TestResult::from_bool(res == Value::Bool(true))
    }

    #[quickcheck]
    fn prop_not_equal_irreflexive(a: i32) -> TestResult {
        if !is_parseable(a) {
            return TestResult::discard();
        }

        let mut interp = Interpreter::new();
        let expr = format!("{} != {}\n", a, a);
        let res = interp.run(&expr).unwrap();

        TestResult::from_bool(res == Value::Bool(false))
    }

    #[quickcheck]
    fn prop_not_equal_symmetric(a: i32, b: i32) -> TestResult {
        if !is_parseable(a) || !is_parseable(b) {
            return TestResult::discard();
        }

        let mut interp = Interpreter::new();

        let expr1 = format!("{} != {}\n", a, b);
        let expr2 = format!("{} != {}\n", b, a);

        let res1 = interp.run(&expr1).unwrap();
        let res2 = interp.run(&expr2).unwrap();

        TestResult::from_bool(res1 == res2)
    }

    #[quickcheck]
    fn prop_less_eq_and_greater_eq_mirror(a: i32, b: i32) -> TestResult {
        if !is_parseable(a) || !is_parseable(b) {
            return TestResult::discard();
        }

        let mut interp = Interpreter::new();

        let expr1 = format!("{} <= {}\n", a, b);
        let expr2 = format!("{} >= {}\n", b, a);

        let res1 = interp.run(&expr1).unwrap();
        let res2 = interp.run(&expr2).unwrap();

        TestResult::from_bool(res1 == res2)
    }

    #[quickcheck]
    fn prop_less_than_or_equal_transitive(a: i32, b: i32, c: i32) -> TestResult {
        if !is_parseable(a) || !is_parseable(b) || !is_parseable(c) {
            return TestResult::discard();
        }

        if a <= b && b <= c {
            let mut interp = Interpreter::new();
            let expr = format!("{} <= {}\n", a, c);
            let res = interp.run(&expr).unwrap();

            TestResult::from_bool(res == Value::Bool(true))
        } else {
            TestResult::discard()
        }
    }

    #[quickcheck]
    fn prop_not_false_is_true() -> TestResult {
        let mut interp = Interpreter::new();
        let expr = "!false\n";
        let res = interp.run(&expr).unwrap();

        TestResult::from_bool(res == Value::Bool(true))
    }

    #[quickcheck]
    fn prop_not_true_is_false() -> TestResult {
        let mut interp = Interpreter::new();
        let expr = "!true\n";
        let res = interp.run(&expr).unwrap();

        TestResult::from_bool(res == Value::Bool(false))
    }

    #[quickcheck]
    fn prop_not_double_negation(a: bool) -> TestResult {
        let mut interp = Interpreter::new();
        let expr = format!("!!{}\n", a);
        let res = interp.run(&expr).unwrap();

        TestResult::from_bool(res == Value::Bool(a))
    }

    #[quickcheck]
    fn prop_and_commutative(a: bool, b: bool) -> TestResult {
        let mut interp = Interpreter::new();

        let expr1 = format!("{} & {}\n", a, b);
        let expr2 = format!("{} & {}\n", b, a);

        let res1 = interp.run(&expr1).unwrap();
        let res2 = interp.run(&expr2).unwrap();

        TestResult::from_bool(res1 == res2)
    }

    #[quickcheck]
    fn prop_and_identity(a: bool) -> TestResult {
        let mut interp = Interpreter::new();
        let expr = format!("true & {}\n", a);
        let res = interp.run(&expr).unwrap();

        TestResult::from_bool(res == Value::Bool(a))
    }

    #[quickcheck]
    fn prop_and_idempotent(a: bool) -> TestResult {
        let mut interp = Interpreter::new();
        let expr = format!("{} & {}\n", a, a);
        let res = interp.run(&expr).unwrap();

        TestResult::from_bool(res == Value::Bool(a))
    }

    #[quickcheck]
    fn prop_or_commutative(a: bool, b: bool) -> TestResult {
        let mut interp = Interpreter::new();

        let expr1 = format!("{} | {}\n", a, b);
        let expr2 = format!("{} | {}\n", b, a);

        let res1 = interp.run(&expr1).unwrap();
        let res2 = interp.run(&expr2).unwrap();

        TestResult::from_bool(res1 == res2)
    }

    #[quickcheck]
    fn prop_or_identity(a: bool) -> TestResult {
        let mut interp = Interpreter::new();
        let expr = format!("false | {}\n", a);
        let res = interp.run(&expr).unwrap();

        TestResult::from_bool(res == Value::Bool(a))
    }

    #[quickcheck]
    fn prop_or_idempotent(a: bool) -> TestResult {
        let mut interp = Interpreter::new();
        let expr = format!("{} | {}\n", a, a);
        let res = interp.run(&expr).unwrap();

        TestResult::from_bool(res == Value::Bool(a))
    }

    #[quickcheck]
    fn prop_greater_than_irreflexive(a: i32) -> TestResult {
        if !is_parseable(a) {
            return TestResult::discard();
        }

        let mut interp = Interpreter::new();
        let expr = format!("{} > {}\n", a, a);
        let res = interp.run(&expr).unwrap();

        TestResult::from_bool(res == Value::Bool(false))
    }

    #[quickcheck]
    fn prop_less_than_irreflexive(a: i32) -> TestResult {
        if !is_parseable(a) {
            return TestResult::discard();
        }

        let mut interp = Interpreter::new();
        let expr = format!("{} < {}\n", a, a);
        let res = interp.run(&expr).unwrap();

        TestResult::from_bool(res == Value::Bool(false))
    }

    #[quickcheck]
    fn prop_greater_less_mirror(a: i32, b: i32) -> TestResult {
        if !is_parseable(a) || !is_parseable(b) {
            return TestResult::discard();
        }

        let mut interp = Interpreter::new();

        let expr1 = format!("{} > {}\n", a, b);
        let expr2 = format!("{} < {}\n", b, a);

        let res1 = interp.run(&expr1).unwrap();
        let res2 = interp.run(&expr2).unwrap();

        TestResult::from_bool(res1 == res2)
    }

    #[quickcheck]
    fn prop_de_morgans_and(a: bool, b: bool) -> TestResult {
        let mut interp = Interpreter::new();

        let expr1 = format!("!({} & {})\n", a, b);
        let expr2 = format!("!{} | !{}\n", a, b);

        let res1 = interp.run(&expr1).unwrap();
        let res2 = interp.run(&expr2).unwrap();

        TestResult::from_bool(res1 == res2)
    }

    #[quickcheck]
    fn prop_de_morgans_or(a: bool, b: bool) -> TestResult {
        let mut interp = Interpreter::new();

        let expr1 = format!("!({} | {})\n", a, b);
        let expr2 = format!("!{} & !{}\n", a, b);

        let res1 = interp.run(&expr1).unwrap();
        let res2 = interp.run(&expr2).unwrap();

        TestResult::from_bool(res1 == res2)
    }
}
