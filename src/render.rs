use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::rect::Rect;
use sdl2::ttf::Font;
use sdl2::gfx::primitives::DrawRenderer;
use crate::game::GameOfLife;

impl GameOfLife {
    pub fn draw(&self, canvas: &mut Canvas<Window>) {
        canvas.set_draw_color(Color::RGB(10, 20, 30));
        canvas.clear();

        for (i, &val) in self.pxl_vec.iter().enumerate() {
            let x = (i % self.a_width as usize) as i32 * self.pixel_edge_size as i32 + (self.pixel_edge_size / 2) as i32;
            let y = (i / self.a_width as usize) as i32 * self.pixel_edge_size as i32 + (self.pixel_edge_size / 2) as i32;

            let color_idx = (val * 255.0).clamp(0.0, 255.0) as usize;
            let color = self.colors[color_idx];

            if self.smooth_edges {
                let radius = (self.pixel_edge_size as f32 * 0.5) as i16;
                let _ = canvas.filled_circle(x as i16, y as i16, radius, color);
            } else {
                canvas.set_draw_color(color);
                let _ = canvas.fill_rect(Rect::new(
                    x - (self.pixel_edge_size / 2) as i32,
                    y - (self.pixel_edge_size / 2) as i32,
                    self.pixel_edge_size,
                    self.pixel_edge_size,
                ));
            }
        }

        canvas.present();
    }

    pub fn update_info_window(&self, font: &Font) {
        if let Some(info_window) = &self.info_window {
            let mut info_canvas = info_window.clone().into_canvas().build().unwrap();
            info_canvas.set_draw_color(Color::RGB(20, 20, 20));
            info_canvas.clear();

            let texture_creator = info_canvas.texture_creator();
            let text_lines = vec![
                format!("FPS: {:.2}", self.fps),
                format!("Generation: {}", self.generation),
                format!("Pixel Size: {}", self.pixel_edge_size),
                format!("Update Frequency: {:.2}", self.update_freq),
                format!("Kernel Radius: {}", self.kernel_rad),
                format!("Bell M: {:.2}", self.bell_m),
                format!("Bell S: {:.2}", self.bell_s),
                format!("Smooth Edges: {}", if self.smooth_edges { "On" } else { "Off" }),
                format!("Gradient Index: {}", self.gradient_idx + 1),
                format!("Running: {}", if self.running { "Yes" } else { "No" }),
                format!("Noise Enabled: {}", if self.noise_enabled { "Yes" } else { "No" }),
                format!("Noise Intensity: {:.2}", self.noise_intensity),
            ];

            let mut y_offset = 10;
            let line_height = 30;

            for line in text_lines {
                let surface = font.render(&line)
                    .blended(Color::RGB(255, 255, 255))
                    .map_err(|e| e.to_string()).unwrap();

                let texture = texture_creator.create_texture_from_surface(&surface)
                    .map_err(|e| e.to_string()).unwrap();

                let target = Rect::new(10, y_offset, surface.width(), surface.height());
                let _ = info_canvas.copy(&texture, None, Some(target));

                y_offset += line_height;
            }

            info_canvas.present();
        }
    }
}

