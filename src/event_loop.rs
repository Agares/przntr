use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::Sdl;
use std::time::Duration;

pub struct EventLoop<'a> {
    sdl: &'a Sdl,
    onloops: Vec<&'a mut dyn OnLoop>,
}

pub trait OnLoop {
    fn run(&mut self) -> Result<(), String>;
}

impl<'a> EventLoop<'a> {
    pub fn new(sdl: &'a Sdl, onloops: Vec<&'a mut dyn OnLoop>) -> Self {
        Self { sdl, onloops }
    }

    pub fn run(&mut self) {
        let mut event_pump = self.sdl.event_pump().unwrap();

        'running: loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. }
                    | Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => break 'running,
                    Event::KeyDown {
                        keycode: Some(keycode),
                        ..
                    } => println!("Keydown: {}", keycode),
                    _ => {}
                }
            }

            for item in &mut self.onloops {
                if item.run().is_err() {
                    println!("OnLoop failed!"); // todo more detailed message, actual logging
                }
            }

            // todo implement the FPS limit correctly
            ::std::thread::sleep(Duration::new(0, 1_000_000_000_u32 / 60));
        }
    }
}
