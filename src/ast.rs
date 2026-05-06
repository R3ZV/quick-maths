use std::collections::HashMap;
use std::fmt;

use crate::common::Value;
use crate::error::Error;
use crate::parser::{BinaryOp, UnaryOp};

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Math(MathExpr),
    Assign(String, MathExpr),
}

#[derive(Clone, Debug, PartialEq)]
pub enum MathExpr {
    Val(Value),
    Var(String),
    BinOp(BinaryOp, Box<MathExpr>, Box<MathExpr>),
    UnaryOp(UnaryOp, Box<MathExpr>),
}

impl fmt::Display for MathExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MathExpr::Val(val) => write!(f, "{}", val),
            MathExpr::Var(var) => write!(f, "{}", var),
            MathExpr::UnaryOp(op, expr) => write!(f, "{}{}", op, expr),
            MathExpr::BinOp(op, left, right) => write!(f, "({} {} {})", left, op, right),
        }
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expr::Math(m) => write!(f, "{}", m),
            Expr::Assign(var, m) => write!(f, "{} = {}", var, m),
        }
    }
}

impl Expr {
    pub fn eval(self, pgm_state: &mut HashMap<String, Value>) -> Result<Value, Error> {
        match self {
            Expr::Math(m_expr) => m_expr.eval(pgm_state),
            Expr::Assign(var, m_expr) => {
                let right = m_expr.eval(pgm_state)?;
                pgm_state.insert(var, right);
                Ok(right)
            }
        }
    }
}

impl MathExpr {
    pub fn eval(&self, pgm_state: &HashMap<String, Value>) -> Result<Value, Error> {
        match self {
            MathExpr::Val(val) => Ok(*val),
            MathExpr::Var(var) => pgm_state.get(var).copied().ok_or(Error::UndeclaredVar),
            MathExpr::BinOp(op, e1, e2) => {
                let v1 = e1.eval(pgm_state)?;
                let v2 = e2.eval(pgm_state)?;
                op.apply(v1, v2)
            }
            MathExpr::UnaryOp(op, e) => op.apply(e.eval(pgm_state)?),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{lexer::Lexer, parser::Parser};

    use super::*;
    use quickcheck::{Arbitrary, Gen, empty_shrinker};
    use quickcheck_macros::quickcheck;
    use std::collections::HashMap;

    fn random_var_name(g: &mut Gen) -> String {
        let names = ["x", "y", "a", "b", "counter", "result"];
        g.choose(&names).unwrap().to_string()
    }

    impl Arbitrary for UnaryOp {
        fn arbitrary(g: &mut Gen) -> Self {
            let ops = [UnaryOp::Not, UnaryOp::Minus];
            *g.choose(&ops).unwrap()
        }
    }

    impl Arbitrary for BinaryOp {
        fn arbitrary(g: &mut Gen) -> Self {
            let ops = [
                BinaryOp::Plus,
                BinaryOp::Minus,
                BinaryOp::Mull,
                BinaryOp::Div,
                BinaryOp::Less,
                BinaryOp::LessEq,
                BinaryOp::Greater,
                BinaryOp::GreaterEq,
                BinaryOp::Equal,
                BinaryOp::NotEqual,
                BinaryOp::And,
                BinaryOp::Or,
            ];
            *g.choose(&ops).unwrap()
        }
    }

    impl Arbitrary for Value {
        fn arbitrary(g: &mut Gen) -> Self {
            if bool::arbitrary(g) {
                let val = (u32::arbitrary(g) % (i32::MAX as u32 + 1)) as i32;
                Value::Int(val)
            } else {
                Value::Bool(bool::arbitrary(g))
            }
        }
    }

    impl Arbitrary for MathExpr {
        fn arbitrary(g: &mut Gen) -> Self {
            let size = g.size();
            gen_math_expr(g, size)
        }
        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            empty_shrinker()
        }
    }

    fn gen_math_expr(g: &mut Gen, size: usize) -> MathExpr {
        if size == 0 {
            // Base cases: Val or Var
            if bool::arbitrary(g) {
                MathExpr::Val(Value::arbitrary(g))
            } else {
                MathExpr::Var(random_var_name(g))
            }
        } else {
            let sub_size = size / 2;
            let choice = usize::arbitrary(g) % 4;

            match choice {
                0 => MathExpr::Val(Value::arbitrary(g)),
                1 => MathExpr::Var(random_var_name(g)),
                2 => MathExpr::UnaryOp(UnaryOp::arbitrary(g), Box::new(gen_math_expr(g, sub_size))),
                _ => MathExpr::BinOp(
                    BinaryOp::arbitrary(g),
                    Box::new(gen_math_expr(g, sub_size)),
                    Box::new(gen_math_expr(g, sub_size)),
                ),
            }
        }
    }

    impl Arbitrary for Expr {
        fn arbitrary(g: &mut Gen) -> Self {
            // Either a MathExpr or an Assignment
            if bool::arbitrary(g) {
                Expr::Math(MathExpr::arbitrary(g))
            } else {
                Expr::Assign(random_var_name(g), MathExpr::arbitrary(g))
            }
        }
        fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
            empty_shrinker()
        }
    }

    #[quickcheck]
    fn prop_eval_is_deterministic(expr: MathExpr, env_data: Vec<(String, Value)>) -> bool {
        let env: HashMap<String, Value> = env_data.into_iter().collect();

        let res1 = expr.eval(&env);
        let res2 = expr.eval(&env);

        format!("{:?}", res1) == format!("{:?}", res2)
    }

    #[quickcheck]
    fn prop_assignment_updates_env(
        var: String,
        m_expr: MathExpr,
        env_data: Vec<(String, Value)>,
    ) -> bool {
        let mut env: HashMap<String, Value> = env_data.into_iter().collect();

        let assign_expr = Expr::Assign(var.clone(), m_expr.clone());
        let expected_val = m_expr.eval(&env);
        let assign_result = assign_expr.eval(&mut env);

        match (expected_val, assign_result) {
            (Ok(val1), Ok(val2)) => {
                let returns_correctly = val1 == val2;
                let env_updated = env.get(&var) == Some(&val1);

                returns_correctly && env_updated
            }
            (Err(_), Err(_)) => true,
            _ => false,
        }
    }

    #[quickcheck]
    fn prop_double_negation_int(v: i32) -> quickcheck::TestResult {
        if v == i32::MIN {
            return quickcheck::TestResult::discard();
        }

        let env = HashMap::new();
        let expr = MathExpr::UnaryOp(
            UnaryOp::Minus,
            Box::new(MathExpr::UnaryOp(
                UnaryOp::Minus,
                Box::new(MathExpr::Val(Value::Int(v))),
            )),
        );

        let res = expr.eval(&env).unwrap();
        quickcheck::TestResult::from_bool(res == Value::Int(v))
    }
    #[quickcheck]
    fn prop_arithmetic_type_mismatch(val: i32, b: bool) -> bool {
        let env = HashMap::new();

        // AST equivalent of: val + b
        let bad_expr = MathExpr::BinOp(
            BinaryOp::Plus,
            Box::new(MathExpr::Val(Value::Int(val))),
            Box::new(MathExpr::Val(Value::Bool(b))),
        );

        matches!(bad_expr.eval(&env), Err(Error::TypeMismatch))
    }

    #[quickcheck]
    fn prop_logical_type_mismatch(val: i32, b: bool) -> bool {
        let env = HashMap::new();

        // AST equivalent of: b & val
        let bad_expr = MathExpr::BinOp(
            BinaryOp::And,
            Box::new(MathExpr::Val(Value::Bool(b))),
            Box::new(MathExpr::Val(Value::Int(val))),
        );

        matches!(bad_expr.eval(&env), Err(Error::TypeMismatch))
    }

    #[quickcheck]
    fn prop_comparison_type_mismatch(val: i32, b: bool) -> bool {
        let env = HashMap::new();

        // AST equivalent of: val > b
        let bad_expr = MathExpr::BinOp(
            BinaryOp::Greater,
            Box::new(MathExpr::Val(Value::Int(val))),
            Box::new(MathExpr::Val(Value::Bool(b))),
        );

        matches!(bad_expr.eval(&env), Err(Error::TypeMismatch))
    }

    #[quickcheck]
    fn prop_unary_not_type_mismatch(val: i32) -> bool {
        let env = HashMap::new();

        // AST equivalent of: !val
        let bad_expr = MathExpr::UnaryOp(UnaryOp::Not, Box::new(MathExpr::Val(Value::Int(val))));

        matches!(bad_expr.eval(&env), Err(Error::TypeMismatch))
    }

    #[quickcheck]
    fn prop_variable_type_mismatch(val: i32, b: bool) -> bool {
        let mut env = HashMap::new();
        env.insert("my_var".to_string(), Value::Bool(b));

        // AST equivalent of: val - my_var
        // Where my_var is a type that doesn't support -
        let bad_expr = MathExpr::BinOp(
            BinaryOp::Minus,
            Box::new(MathExpr::Val(Value::Int(val))),
            Box::new(MathExpr::Var("my_var".to_string())),
        );

        matches!(bad_expr.eval(&env), Err(Error::TypeMismatch))
    }

    #[quickcheck]
    fn prop_round_trip_parser(original_expr: Expr) -> bool {
        let source_code = format!("{}\n", original_expr.to_string());
        let mut lexer = Lexer::new(&source_code);
        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(e) => {
                println!("Lexer failed on: '{}' with error: {:?}", source_code, e);
                return false;
            }
        };

        let mut parser = Parser::new(tokens);
        let parsed_expr = match parser.parse_expr() {
            Ok(e) => e,
            Err(e) => {
                println!("Parser failed on: '{}' with error: {:?}", source_code, e);
                return false;
            }
        };

        if original_expr != parsed_expr {
            println!("Mismatch!");
            println!("Original: {:?}", original_expr);
            println!("String:   {}", source_code);
            println!("Parsed:   {:?}", parsed_expr);
            return false;
        }

        true
    }
}
