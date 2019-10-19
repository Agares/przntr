#![deny(unsafe_code)]

use crate::parsing::parser::Parser;
use parsing::tokenizer::Tokenizer;
use rendering::renderer::Renderer;
use std::fs;

mod parsing;
mod presentation;
mod rendering;

fn main() {
    let mut args = std::env::args();

    args.next();

    let file = fs::read_to_string(args.next().unwrap()).unwrap();

    let mut t = Tokenizer::new(&file);
    let mut p = Parser::new(&mut t);
    let mut r = rendering::renderer::SDL2Renderer::new();

    let presentation = p.parse().unwrap();
    println!("{:?}", presentation);
    r.render(&presentation);
}
