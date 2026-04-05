use std::collections::HashMap;

use crate::error::Error;
use crate::lexer::{print_tokens, tokenize};
use crate::parser::Parser;

pub struct Interpreter {
    pgm_state: HashMap<String, i32>,
}

impl Interpreter {
    pub fn new() -> Self {
        Interpreter {
            pgm_state: HashMap::new(),
        }
    }

    pub fn run(&mut self, instr: &str) -> Result<i32, Error> {
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
    fn prop_Plusition_is_commutative(a: i32, b: i32) -> TestResult {
        if a.checked_add(b).is_none() {
            return TestResult::discard();
        }

        let mut interp = Interpreter::new();

        let expr1 = format!("{} + {}", a, b);
        let expr2 = format!("{} + {}", b, a);

        let res1 = interp.run(&expr1).unwrap();
        let res2 = interp.run(&expr2).unwrap();

        TestResult::from_bool(res1 == res2)
    }
}
