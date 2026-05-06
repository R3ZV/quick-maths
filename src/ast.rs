use std::collections::HashMap;

use crate::common::Value;
use crate::error::Error;
use crate::parser::{BinaryOp, UnaryOp};

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Math(MathExpr),
    Assign(String, MathExpr),
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

#[derive(Clone, Debug, PartialEq)]
pub enum MathExpr {
    Val(Value),
    Var(String),
    BinOp(BinaryOp, Box<MathExpr>, Box<MathExpr>),
    UnaryOp(UnaryOp, Box<MathExpr>),
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
    use super::*; // Import your AST and enums
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
                Value::Int(i32::arbitrary(g))
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
            (Err(_), Err(_)) => {
                true
            }
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
}
