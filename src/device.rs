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
    scale: u32
}

impl Device {

    pub fn start(scale: u32) -> Device {
        let opengl = OpenGL::V3_2;

        let window: Window = WindowSettings::new("CHIP-8", [64 * scale, 32 * scale])
            .graphics_api(opengl)
            .resizable(false)
            .exit_on_esc(true)
            .build()
            .unwrap();

        Device {
            gl: GlGraphics::new(opengl),
            window,
            scale
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

            graphics::image(&texture, c.transform.scale(self.scale as f64, self.scale as f64), gl);
        });
    }

}
