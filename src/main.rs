use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::Sdl;
use sdl2::ttf::Font;
use sdl2::gfx::primitives::DrawRenderer;
use rand::{Rng, rngs::StdRng, SeedableRng};
use rayon::prelude::*;
use std::time::{Instant, Duration};
use colorgrad::{self, Gradient};
use colorgrad::preset::{viridis, inferno, plasma, magma, rainbow};
use rayon::ThreadPoolBuilder;

const DEFAULT_PIXEL_EDGE_SIZE: u32 = 5;
const DEFAULT_UPDATE_FREQ: f64 = 10.0;
const DEFAULT_KERNEL_RAD: u32 = 13;
const DEFAULT_BELL_M: f64 = 0.12;
const DEFAULT_BELL_S: f64 = 0.015;
const DEFAULT_INFO_BAR_HEIGHT: u32 = 100;

fn growth(neighbours: f64, bell_m: f64, bell_s: f64) -> f64 {
    bell(neighbours, bell_m, bell_s) * 2.0 - 1.0
}

fn bell(x: f64, m: f64, s: f64) -> f64 {
    f64::exp(-((x - m) / s).powi(2) / 2.0)
}

struct GameOfLife {
    pxl_vec: Vec<f64>,
    last_update: Instant,
    running: bool,
    fps: f32,
    generation: u64,
    colors: Vec<Color>, // Precomputed colors for faster lookup
    smooth_edges: bool,
    update_skip_counter: usize,
    width: u32,
    height: u32,
    a_width: u32,
    a_height: u32,
    pixel_edge_size: u32,
    update_freq: f64,
    kernel_rad: u32,
    bell_m: f64,
    bell_s: f64,
    info_bar_height: u32,
    gradient_idx: usize,
    gradients: Vec<Box<dyn Gradient>>,
    info_window: Option<Window>,
    mouse_down: bool, // Track if mouse button is held down
}

impl GameOfLife {
    pub fn new(width: u32, height: u32, pixel_edge_size: u32, video_subsystem: &sdl2::VideoSubsystem) -> Self {
        let a_width = width / pixel_edge_size;
        let a_height = height / pixel_edge_size;
        let a_size = (a_width * a_height) as usize;

        let mut pxl_vec = vec![0.0; a_size];
        let mut rng = StdRng::seed_from_u64(42);
        pxl_vec.iter_mut().for_each(|i| {
            *i = rng.gen();
        });

        let gradients: Vec<Box<dyn Gradient>> = vec![
            Box::new(viridis()),
            Box::new(inferno()),
            Box::new(plasma()),
            Box::new(magma()),
            Box::new(rainbow()),
        ];
        let gradient = &gradients[0];

        let colors = Self::compute_colors(&**gradient);

        Self {
            pxl_vec,
            last_update: Instant::now(),
            running: false,
            fps: 0.0,
            generation: 0,
            colors,
            smooth_edges: false,
            update_skip_counter: 0,
            width,
            height,
            a_width,
            a_height,
            pixel_edge_size,
            update_freq: DEFAULT_UPDATE_FREQ,
            kernel_rad: DEFAULT_KERNEL_RAD,
            bell_m: DEFAULT_BELL_M,
            bell_s: DEFAULT_BELL_S,
            info_bar_height: DEFAULT_INFO_BAR_HEIGHT,
            gradient_idx: 0,
            gradients,
            info_window: None,
            mouse_down: false,
        }
    }

    fn compute_colors(gradient: &dyn Gradient) -> Vec<Color> {
        (0..=255)
            .map(|i| {
                let c = gradient.at(i as f32 / 255.0).to_rgba8();
                Color::RGBA(c[0], c[1], c[2], 255)
            })
            .collect()
    }

    fn update(&mut self) {
        if self.running && self.update_skip_counter % 10 == 0 && self.last_update.elapsed() >= Duration::from_millis(16) {
            let mut new_pxl_vec = self.pxl_vec.clone();
            new_pxl_vec.par_iter_mut().enumerate().for_each(|(i, val)| {
                let x = (i % self.a_width as usize) as i32;
                let y = (i / self.a_width as usize) as i32;
                let mut neighbours = 0.0;
                let mut count = 0;
                let kernel_radius = self.kernel_rad as i32;

                // Iterate over the neighborhood
                for dy in -kernel_radius..=kernel_radius {
                    for dx in -kernel_radius..=kernel_radius {
                        let nx = x + dx;
                        let ny = y + dy;
                        if nx >= 0 && ny >= 0 && (nx as u32) < self.a_width && (ny as u32) < self.a_height {
                            neighbours += self.pxl_vec[(ny as usize) * self.a_width as usize + (nx as usize)];
                            count += 1;
                        }
                    }
                }

                if count > 0 {
                    neighbours /= count as f64;
                }
                *val = (*val + ((1.0 / self.update_freq) * growth(neighbours, self.bell_m, self.bell_s))).clamp(0.0, 1.0);
            });

            self.pxl_vec = new_pxl_vec;
            self.generation += 1;
            self.fps = 1000.0 / (self.last_update.elapsed().as_millis() as f32);
            self.last_update = Instant::now();
        }
        self.update_skip_counter += 1;
    }


    fn add_cells_with_brush(&mut self, mouse_x: i32, mouse_y: i32, brush_radius: i32, state: f64) {
        // Calculate which cells are within the brush radius
        for dy in -brush_radius..=brush_radius {
            for dx in -brush_radius..=brush_radius {
                let nx = mouse_x / self.pixel_edge_size as i32 + dx;
                let ny = mouse_y / self.pixel_edge_size as i32 + dy;

                if nx >= 0 && ny >= 0 && (nx as u32) < self.a_width && (ny as u32) < self.a_height {
                    let distance = ((dx * dx + dy * dy) as f64).sqrt();
                    if distance <= brush_radius as f64 {
                        let index = (ny as usize * self.a_width as usize + nx as usize) as usize;
                        self.pxl_vec[index] = state; // Set the cell state to the given value (1.0 for alive, 0.0 for dead)
                    }
                }
            }
        }
    }

    fn resize(&mut self, new_width: u32, new_height: u32) {
        self.width = new_width;
        self.height = new_height;
        self.a_width = new_width / self.pixel_edge_size;
        self.a_height = new_height / self.pixel_edge_size;
        let new_a_size = (self.a_width * self.a_height) as usize;

        self.pxl_vec = vec![0.0; new_a_size];
        let mut rng = StdRng::seed_from_u64(42);
        self.pxl_vec.iter_mut().for_each(|i| {
            *i = rng.gen();
        });
    }

    fn change_pixel_size(&mut self, delta: i32) {
        let new_pixel_size = (self.pixel_edge_size as i32 + delta).clamp(1, 50) as u32;
        if new_pixel_size != self.pixel_edge_size {
            self.pixel_edge_size = new_pixel_size;
            self.resize(self.width, self.height);
        }
    }

    fn change_parameter(&mut self, param: &str, delta: f64) {
        match param {
            "update_freq" => {
                self.update_freq = (self.update_freq + delta).clamp(1.0, 100.0);
            }
            "kernel_rad" => {
                self.kernel_rad = (self.kernel_rad as f64 + delta).clamp(1.0, 20.0) as u32;
            }
            "bell_m" => {
                self.bell_m = (self.bell_m + delta).clamp(0.01, 1.0);
            }
            "bell_s" => {
                self.bell_s = (self.bell_s + delta).clamp(0.01, 1.0);
            }
            "info_bar_height" => {
                self.info_bar_height = (self.info_bar_height as f64 + delta).clamp(0.0, 200.0) as u32;
            }
            _ => {}
        }
    }

    fn reset_parameters(&mut self) {
        self.pixel_edge_size = DEFAULT_PIXEL_EDGE_SIZE;
        self.update_freq = DEFAULT_UPDATE_FREQ;
        self.kernel_rad = DEFAULT_KERNEL_RAD;
        self.bell_m = DEFAULT_BELL_M;
        self.bell_s = DEFAULT_BELL_S;
        self.info_bar_height = DEFAULT_INFO_BAR_HEIGHT;
        self.resize(self.width, self.height);
    }

    fn switch_gradient(&mut self) {
        self.gradient_idx = (self.gradient_idx + 1) % self.gradients.len();
        self.colors = Self::compute_colors(self.gradients[self.gradient_idx].as_ref());
    }

    fn toggle_info_window(&mut self, video_subsystem: &sdl2::VideoSubsystem) {
        if let Some(_info_window) = &self.info_window {
            self.info_window = None;
        } else {
            let info_window = video_subsystem
                .window("Simulation Info", 500, 400) // Increased size for better readability
                .position_centered()
                .build()
                .unwrap();

            let info_canvas = info_window.into_canvas().build().unwrap();
            self.info_window = Some(info_canvas.into_window());
        }
    }


    fn update_info_window(&self, font: &Font) {
        if let Some(info_window) = &self.info_window {
            let mut info_canvas = <sdl2::video::Window as Clone>::clone(&info_window).into_canvas().build().unwrap();
            info_canvas.set_draw_color(Color::RGB(20, 20, 20)); // Dark background for better contrast
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
            ];

            let mut y_offset = 10;
            let line_height = 30; // Space between lines for better readability

            for line in text_lines {
                let surface = font.render(&line)
                    .blended(Color::RGB(255, 255, 255)) // White text for high contrast
                    .map_err(|e| e.to_string()).unwrap();

                let texture = texture_creator.create_texture_from_surface(&surface)
                    .map_err(|e| e.to_string()).unwrap();

                let target = Rect::new(10, y_offset, surface.width(), surface.height());
                let _ = info_canvas.copy(&texture, None, Some(target));

                y_offset += line_height; // Move down for the next line
            }

            info_canvas.present();
        }
    }


    fn draw(&self, canvas: &mut Canvas<Window>) {
        canvas.set_draw_color(Color::RGB(10, 20, 30));
        canvas.clear();

        // Draw game grid
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
}

fn main() {
    // Limit the number of threads used by Rayon
    ThreadPoolBuilder::new().num_threads(4).build_global().unwrap();

    let sdl_context: Sdl = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();

    let window = video_subsystem.window("Game of Life", 750, 750)
        .position_centered()
        .resizable()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    let font = ttf_context.load_font("/System/Library/Fonts/SFNS.ttf", 16).unwrap();

    let mut game = GameOfLife::new(750, 750, DEFAULT_PIXEL_EDGE_SIZE, &video_subsystem);

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } |
                    Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        break 'running;
                    },
                Event::KeyDown { keycode: Some(Keycode::Space), .. } => {
                    game.running = !game.running;
                },
                Event::KeyDown { keycode: Some(Keycode::H), .. } => {
                    game.toggle_info_window(&video_subsystem);
                },
                Event::KeyDown { keycode: Some(Keycode::Up), .. } => {
                    game.change_pixel_size(1);
                },
                Event::KeyDown { keycode: Some(Keycode::Down), .. } => {
                    game.change_pixel_size(-1);
                },
                Event::KeyDown { keycode: Some(Keycode::S), .. } => {
                    game.smooth_edges = !game.smooth_edges;
                },
                Event::KeyDown { keycode: Some(Keycode::R), .. } => {
                    game.reset_parameters();
                },
                Event::KeyDown { keycode: Some(Keycode::C), .. } => {
                    game.switch_gradient();
                },
                Event::KeyDown { keycode: Some(Keycode::F1), .. } => {
                    game.change_parameter("update_freq", 1.0);
                },
                Event::KeyDown { keycode: Some(Keycode::F2), .. } => {
                    game.change_parameter("update_freq", -1.0);
                },
                Event::KeyDown { keycode: Some(Keycode::F3), .. } => {
                    game.change_parameter("kernel_rad", 1.0);
                },
                Event::KeyDown { keycode: Some(Keycode::F4), .. } => {
                    game.change_parameter("kernel_rad", -1.0);
                },
                Event::KeyDown { keycode: Some(Keycode::F5), .. } => {
                    game.change_parameter("bell_m", 0.01);
                },
                Event::KeyDown { keycode: Some(Keycode::F6), .. } => {
                    game.change_parameter("bell_m", -0.01);
                },
                Event::KeyDown { keycode: Some(Keycode::F7), .. } => {
                    game.change_parameter("bell_s", 0.001);
                },
                Event::KeyDown { keycode: Some(Keycode::F8), .. } => {
                    game.change_parameter("bell_s", -0.001);
                },
                Event::KeyDown { keycode: Some(Keycode::F9), .. } => {
                    game.change_parameter("info_bar_height", 5.0);
                },
                Event::KeyDown { keycode: Some(Keycode::F10), .. } => {
                    game.change_parameter("info_bar_height", -5.0);
                },
                Event::MouseButtonDown { mouse_btn, x, y, .. } => {
                    game.mouse_down = true;
                    let state = match mouse_btn {
                        sdl2::mouse::MouseButton::Left => 1.0,   // Full alive cell
                        sdl2::mouse::MouseButton::Right => 0.0,  // Full dead cell
                        _ => continue,
                    };
                    game.add_cells_with_brush(x, y, 5, state); // Use a brush radius of 5 cells (adjust as needed)
                },
                Event::MouseButtonUp { mouse_btn: sdl2::mouse::MouseButton::Left, .. }
                    | Event::MouseButtonUp { mouse_btn: sdl2::mouse::MouseButton::Right, .. } => {
                        game.mouse_down = false;
                    },
                Event::MouseMotion { x, y, .. } => {
                    if game.mouse_down {
                        game.add_cells_with_brush(x, y, 5, 1.0); // Keep drawing full alive cells on motion if left button is held
                    }
                },

                Event::Window { win_event: sdl2::event::WindowEvent::Resized(new_width, new_height), .. } => {
                    game.resize(new_width as u32, new_height as u32);
                },
                _ => {}
            }
        }

        game.update();
        game.draw(&mut canvas);
        game.update_info_window(&font);
    }
}
