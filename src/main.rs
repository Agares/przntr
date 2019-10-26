#![deny(unsafe_code)]
#![deny(clippy::all, clippy::pedantic, clippy::cargo)]
// todo remove the allows
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::multiple_crate_versions)]

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

    let sdl_context = sdl2::init().expect("Failed to initialize SDL2");
    let sdl_ttf_context = sdl2::ttf::init().expect("Failed to initialize SDL2 ttf");
    let file = fs::read_to_string(
        args.next()
            .expect("Missing argument (path to the presentation)"),
    )
    .expect("Failed to read the presentation file");

    let mut t = Tokenizer::new(&file);
    let mut p = Parser::new(&mut t);

    let presentation = p.parse().expect("Presentation was not parsed correctly");
    let mut r = rendering::renderer::SDL2::new(&sdl_context, &sdl_ttf_context, &presentation);

    let mut ev_loop = EventLoop::new(&sdl_context, vec![&mut r]);
    ev_loop.run();
}
