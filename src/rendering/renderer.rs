use crate::event_loop::OnLoop;
use crate::presentation::Presentation;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::{Texture, WindowCanvas};
use sdl2::surface::Surface;
use sdl2::ttf::{Font, Sdl2TtfContext};
use sdl2::Sdl;

pub struct SDL2Renderer<'a> {
    font: Font<'a, 'a>,
    window_canvas: WindowCanvas,
}

impl<'a> SDL2Renderer<'a> {
    pub fn new(sdl: &'a Sdl, sdl_ttf: &'a Sdl2TtfContext, presentation: &'a Presentation) -> Self {
        let mut window_canvas = sdl
            .video()
            .unwrap()
            .window("some presentation", 800, 600)
            .position_centered()
            .build()
            .unwrap()
            .into_canvas()
            .build()
            .unwrap();

        window_canvas.set_draw_color(Color::RGB(0, 0, 0));
        window_canvas.clear();
        window_canvas.present();

        Self {
            font: sdl_ttf
                .load_font(presentation.style().fonts().first().unwrap().path(), 24)
                .unwrap(),
            window_canvas,
        }
    }

    fn window_center(&self) -> Point {
        Point::new(
            (self.window_canvas.window().size().0 / 2) as i32,
            (self.window_canvas.window().size().1 / 2) as i32,
        )
    }

    fn render_text(&self, text: &str) -> Surface {
        self.font
            .render(text)
            .blended(Color::RGB(0xff, 0x18, 0x85))
            .unwrap()
    }
}

impl<'a> OnLoop for SDL2Renderer<'a> {
    fn run(&mut self) {
        self.window_canvas.clear();

        let txt = self.render_text("test");

        let txt_rect = txt.rect();
        let mut dst_txt_rect = txt_rect;
        dst_txt_rect.center_on(self.window_center());
        let texture_creator = self.window_canvas.texture_creator();
        let texture: Texture = texture_creator.create_texture_from_surface(txt).unwrap();

        self.window_canvas
            .copy(&texture, txt_rect, dst_txt_rect)
            .unwrap();
        self.window_canvas.present();
    }
}
