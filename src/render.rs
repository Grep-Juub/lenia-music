use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::rect::Rect;
use sdl2::ttf::Font;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::mouse::MouseButton;
use sdl2::event::Event;
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

    pub fn handle_slider_events(&mut self, event: &Event) {
        if self.overlay_visible {
            match *event {
                Event::MouseButtonDown { x, y, mouse_btn: MouseButton::Left, .. } => {
                    // Check if the click is within the slider area
                    let slider_y_offsets = [50, 80, 110, 140, 170]; // Example y_offsets for sliders
                    let slider_x = 200;
                    let slider_width = 200;
                    let slider_height = 10;

                    for (i, &slider_y) in slider_y_offsets.iter().enumerate() {
                        if y >= slider_y + 10 && y <= slider_y + 10 + slider_height && x >= slider_x && x <= slider_x + slider_width {
                            let new_value = ((x - slider_x) as f32 / slider_width as f32).clamp(0.0, 1.0);
                            match i {
                                0 => self.update_freq = new_value as f64 * (100.0 - 1.0) + 1.0,
                                1 => self.kernel_rad = (new_value as f64 * (20.0 - 1.0) + 1.0).round() as u32,
                                2 => self.bell_m = new_value as f64 * (1.0 - 0.01) + 0.01,
                                3 => self.bell_s = new_value as f64 * (1.0 - 0.01) + 0.01,
                                4 => self.noise_intensity = new_value as f64 * (1.0 - 0.0) + 0.0,
                                _ => {},
                            }
                        }
                    }
                },
                _ => {},
            }
        }
    }
}

