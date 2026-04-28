// TODO: Add messages to errors!

mod ast;
mod error;
mod interpreter;
mod lexer;
mod parser;
mod common;

use std::io::{Write, stdin, stdout};

use interpreter::Interpreter;

fn print_err(err: &str) {
    println!("[ERR]: {}", err);
}

fn main() {
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
