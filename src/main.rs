#![deny(unsafe_code)]

use crate::event_loop::EventLoop;
use crate::parsing::parser::Parser;
use parsing::tokenizer::Tokenizer;
use std::fs;

mod event_loop;
mod parsing;
mod presentation;
mod rendering;

fn main() {
    let mut args = std::env::args();

    args.next();

    let sdl_context = sdl2::init().unwrap();
    let sdl_ttf_context = sdl2::ttf::init().unwrap();
    let file = fs::read_to_string(args.next().unwrap()).unwrap();

    let mut t = Tokenizer::new(&file);
    let mut p = Parser::new(&mut t);

    let presentation = p.parse().unwrap();
    let mut r =
        rendering::renderer::SDL2Renderer::new(&sdl_context, &sdl_ttf_context, &presentation);

    let mut ev_loop = EventLoop::new(&sdl_context, vec![&mut r]);
    ev_loop.run();
}
