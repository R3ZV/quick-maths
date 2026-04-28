// TODO: Plus messages to errors!
// TODO: Negative values will cause issues
//      - Solution: Handle them in when evaluating
// TODO: Floats will cause issues
//      - Solution let the language parser handle numeric conversion,
//      - Just identify the slice that represents the number
// TODO: Why don't consume and current work with refereces?
// TODO: Make parse functions for operators and parant that accepts a single char
// TODO: Refactor tokenize a bit

mod ast;
mod error;
mod interpreter;
mod lexer;
mod parser;
mod common;

// use lexer::{tokenize, print_tokens};
use std::io::{Write, stdin, stdout};

use interpreter::Interpreter;

fn print_err(err: &str) {
    println!("[ERR]: {}", err);
}

fn main() {
    // let x = "true & false";
    // print_tokens(&tokenize(x).unwrap());
    let mut interpreter: Interpreter = Interpreter::new();
    println!("You are in the quick maths interactive shell!\n");
    loop {
        print!(">> ");

        let mut instr = String::new();
        let _ = stdout().flush();
        stdin()
            .read_line(&mut instr)
            .expect("Did not enter a correct string");

        // println!("[DBG]: {}", instr);

        if instr.trim() == "q" {
            println!("Exiting...");
            return;
        }

        let instr_res = interpreter.run(&instr);

        match instr_res {
            Ok(val) => {
                println!("{}", val)
            }
            Err(_) => print_err("Amogus"),
        }
    }
}
