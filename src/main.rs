use std::fs;
use crate::tokenizer::{Tokenizer, TokenizerResult};

mod tokenizer;

fn main() {
    let mut args = std::env::args();

    args.next();

    let file = fs::read_to_string(args.next().unwrap()).unwrap();

    let mut t = Tokenizer::new(&file);

    loop {
        let tokenizer_result = t.next();
        match tokenizer_result {
            TokenizerResult::Err(err) => panic!("{:?}", err),
            TokenizerResult::End => break,
            TokenizerResult::Ok(token) => {
                println!("{:?}", token)
            }
        }
    }
}
