use glutin_window::OpenGL;
use opengl_graphics::{Filter, GlGraphics, Texture, TextureSettings};
use piston::{
    window::WindowSettings, RenderArgs
};

use graphics::{clear, Transformed};

use glutin_window::GlutinWindow as Window;

pub struct Device {
    gl: GlGraphics,
    pub window: Window,
}

const SCALE: u32 = 16;
const WIDTH: u32 = SCALE * 64;
const HEIGHT: u32 = SCALE * 32;

impl Device {

    pub fn start() -> Device {
        let opengl = OpenGL::V3_2;

        let window: Window = WindowSettings::new("CHIP-8", [WIDTH, HEIGHT])
            .graphics_api(opengl)
            .resizable(false)
            .exit_on_esc(true)
            .build()
            .unwrap();

        Device {
            gl: GlGraphics::new(opengl),
            window,
        }
    }

    pub fn render(&mut self, args: &RenderArgs, video: [u8; 64*32]) {

        //let image = Image::new().rect([0.0, 0.0, WIDTH as f64, HEIGHT as f64]);
        let mut setting = TextureSettings::new();
        setting.set_filter(Filter::Nearest);
        let texture = Texture::from_memory_alpha(&video, 64, 32, &setting).unwrap();

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear([0.0, 0.0, 0.0, 1.0], gl);

            graphics::image(&texture, c.transform.scale(SCALE as f64, SCALE as f64), gl);
        });
    }

}
