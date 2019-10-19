use crate::presentation::Presentation;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::Texture;
use std::time::Duration;

pub trait Renderer {
    fn render(&mut self, presentation: &Presentation);
}

pub struct SDL2Renderer {}

impl SDL2Renderer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Renderer for SDL2Renderer {
    fn render(&mut self, presentation: &Presentation) {
        let sdl_context = sdl2::init().unwrap();
        let sdl_ttf_context = sdl2::ttf::init().unwrap();
        let font = sdl_ttf_context
            .load_font(presentation.style().fonts().first().unwrap().path(), 24)
            .unwrap();

        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("rust-sdl2 demo", 800, 600)
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();
        let window_center = Point::new(
            (canvas.window().size().0 / 2) as i32,
            (canvas.window().size().1 / 2) as i32,
        );

        let txt = font
            .render("test")
            .blended(Color::RGB(0xff, 0x18, 0x85))
            .unwrap();
        let txt_rect = txt.rect();
        let mut dst_txt_rect = txt_rect;
        dst_txt_rect.center_on(window_center);
        let texture_creator = canvas.texture_creator();
        let texture: Texture = texture_creator.create_texture_from_surface(txt).unwrap();

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        let mut event_pump = sdl_context.event_pump().unwrap();

        'running: loop {
            canvas.clear();

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

            canvas.copy(&texture, txt_rect, dst_txt_rect).unwrap();
            canvas.present();

            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
        }
    }
}
