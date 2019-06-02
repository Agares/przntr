use crate::parsing::parser::Parser;
use parsing::tokenizer::Tokenizer;
use std::fs;

mod parsing;

fn main() {
    let mut args = std::env::args();

    args.next();

    let file = fs::read_to_string(args.next().unwrap()).unwrap();

    let mut t = Tokenizer::new(&file);
    let mut p = Parser::new(&mut t);

    println!("{:?}", p.parse());
}
