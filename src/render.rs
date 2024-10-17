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

    pub fn update_info_window(&mut self, font: &Font) {
        if let Some(info_window) = &self.info_window {
            let mut info_canvas = info_window.clone().into_canvas().build().unwrap();
            info_canvas.set_draw_color(Color::RGB(20, 20, 20));
            info_canvas.clear();

            let texture_creator = info_canvas.texture_creator();
            let text_lines = vec![
                format!("FPS: {:.2}", self.fps),
                format!("Generation: {}", self.generation),
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

            // Adding sliders for adjusting parameters
            self.draw_slider(&mut info_canvas, &font, "Update Frequency", self.update_freq as f32, 1.0, 100.0, y_offset);
            y_offset += line_height;
            self.draw_slider(&mut info_canvas, &font, "Kernel Radius", self.kernel_rad as f32, 1.0, 20.0, y_offset);
            y_offset += line_height;
            self.draw_slider(&mut info_canvas, &font, "Bell M", self.bell_m as f32, 0.01, 1.0, y_offset);
            y_offset += line_height;
            self.draw_slider(&mut info_canvas, &font, "Bell S", self.bell_s as f32, 0.01, 1.0, y_offset);
            y_offset += line_height;
            self.draw_slider(&mut info_canvas, &font, "Noise Intensity", self.noise_intensity as f32, 0.0, 1.0, y_offset);

            info_canvas.present();
        }
    }

    fn draw_slider(&self, canvas: &mut Canvas<Window>, font: &Font, label: &str, value: f32, min: f32, max: f32, y_offset: i32) {
        let texture_creator = canvas.texture_creator();
        let label_surface = font.render(&format!("{}: {:.2}", label, value))
            .blended(Color::RGB(255, 255, 255))
            .map_err(|e| e.to_string()).unwrap();
        let label_texture = texture_creator.create_texture_from_surface(&label_surface)
            .map_err(|e| e.to_string()).unwrap();

        let label_target = Rect::new(10, y_offset, label_surface.width(), label_surface.height());
        let _ = canvas.copy(&label_texture, None, Some(label_target));

        // Draw the slider background
        let slider_x = 200;
        let slider_y = y_offset + 10;
        let slider_width = 200;
        let slider_height = 10;
        canvas.set_draw_color(Color::RGB(100, 100, 100));
        let _ = canvas.fill_rect(Rect::new(slider_x, slider_y, slider_width, slider_height));

        // Draw the slider knob
        let knob_x = slider_x + ((value - min) / (max - min) * slider_width as f32) as i32;
        canvas.set_draw_color(Color::RGB(200, 200, 200));
        let _ = canvas.fill_rect(Rect::new(knob_x - 5, slider_y - 5, 10, 20));
    }

    pub fn handle_slider_events(&mut self, event: &Event) {
        if let Some(_info_window) = &self.info_window {
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

