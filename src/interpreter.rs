use std::collections::HashMap;

use crate::error::Error;
use crate::common::{Value};
use crate::lexer::{print_tokens, tokenize};
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
        let tokens = tokenize(instr)?;
        print_tokens(&tokens);

        let mut parser: Parser = Parser::new(tokens);
        let expr = parser.parse_expr()?;
        println!("[DBG]: Passed parser!");

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
        let _ = tokenize(&input);
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

        let expr1 = format!("{} + {}", a, b);
        let expr2 = format!("{} + {}", b, a);

        let res1 = interp.run(&expr1).unwrap();
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

        let expr1 = format!("({} + {}) + {}", a, b, c);
        let expr2 = format!("{} + ({} + {})", a, b, c);

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
        let expr1 = format!("{} + 0", a);
        let expr2 = format!("0 + {}", a);

        let res1 = interp.run(&expr1).unwrap();
        let res2 = interp.run(&expr2).unwrap();

        TestResult::from_bool(res1 == a && res2 == a)
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

        let expr1 = format!("{} * {}", a, b);
        let expr2 = format!("{} * {}", b, a);

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

        let expr1 = format!("({} * {}) * {}", a, b, c);
        let expr2 = format!("{} * ({} * {})", a, b, c);

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

        let expr1 = format!("{} * ({} + {})", a, b, c);
        let expr2 = format!("{} * {} + {} * {}", a, b, a, c);

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
        let expr = format!("{} - {}", a, a);
        let res = interp.run(&expr).unwrap();

        TestResult::from_bool(res == 0)
    }

    #[quickcheck]
    fn prop_division_by_self_is_one(a: i32) -> TestResult {
        // Discard 0 to avoid Division By Zero panics
        if a == 0 || !is_parseable(a) {
            return TestResult::discard();
        }

        let mut interp = Interpreter::new();
        let expr = format!("{} / {}", a, a);
        let res = interp.run(&expr).unwrap();

        TestResult::from_bool(res == 1)
    }
}
