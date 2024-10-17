use sdl2::pixels::Color;
use sdl2::video::Window;
use rand::{Rng, rngs::StdRng, SeedableRng};
use rayon::prelude::*;
use std::time::{Instant, Duration};
use colorgrad::{self, Gradient};
use colorgrad::preset::{viridis, inferno, plasma, magma, rainbow};
use crate::utils::{growth};

pub const DEFAULT_PIXEL_EDGE_SIZE: u32 = 5;
pub const DEFAULT_UPDATE_FREQ: f64 = 10.0;
pub const DEFAULT_KERNEL_RAD: u32 = 13;
pub const DEFAULT_BELL_M: f64 = 0.12;
pub const DEFAULT_BELL_S: f64 = 0.015;
pub const DEFAULT_INFO_BAR_HEIGHT: u32 = 100;
pub const DEFAULT_NOISE_INTENSITY: f64 = 0.1;

pub struct GameOfLife {
    pub pxl_vec: Vec<f64>,
    pub last_update: Instant,
    pub running: bool,
    pub fps: f32,
    pub generation: u64,
    pub colors: Vec<Color>, // Precomputed colors for faster lookup
    pub smooth_edges: bool,
    pub update_skip_counter: usize,
    pub width: u32,
    pub height: u32,
    pub a_width: u32,
    pub a_height: u32,
    pub pixel_edge_size: u32,
    pub update_freq: f64,
    pub kernel_rad: u32,
    pub bell_m: f64,
    pub bell_s: f64,
    pub info_bar_height: u32,
    pub gradient_idx: usize,
    pub gradients: Vec<Box<dyn Gradient>>,
    pub info_window: Option<Window>,
    pub mouse_down: bool, // Track if mouse button is held down
    pub noise_intensity: f64,
    pub noise_enabled: bool,
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
            noise_intensity: DEFAULT_NOISE_INTENSITY,
            gradient_idx: 0,
            gradients,
            info_window: None,
            mouse_down: false,
            noise_enabled: true,
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

    pub fn update(&mut self) {
        if self.running && self.update_skip_counter % 10 == 0 && self.last_update.elapsed() >= Duration::from_millis(16) {
            let mut new_pxl_vec = self.pxl_vec.clone();
            new_pxl_vec.par_iter_mut().enumerate().for_each(|(i, val)| {
                let x = (i % self.a_width as usize) as i32;
                let y = (i / self.a_width as usize) as i32;
                let mut neighbours = 0.0;
                let mut count = 0;
                let kernel_radius = self.kernel_rad as i32;

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

                let mut rng = StdRng::from_entropy();
                let noise = if self.noise_enabled {
                    rng.gen_range(-self.noise_intensity..self.noise_intensity)
                } else {
                    0.0
                };
                *val = (*val + noise + ((1.0 / self.update_freq) * growth(neighbours, self.bell_m, self.bell_s))).clamp(0.0, 1.0);
            });

            self.pxl_vec = new_pxl_vec;
            self.generation += 1;
            self.fps = 1000.0 / (self.last_update.elapsed().as_millis() as f32);
            self.last_update = Instant::now();
        }
        self.update_skip_counter += 1;
    }

    pub fn add_cells_with_brush(&mut self, mouse_x: i32, mouse_y: i32, brush_radius: i32, state: f64) {
        for dy in -brush_radius..=brush_radius {
            for dx in -brush_radius..=brush_radius {
                let nx = mouse_x / self.pixel_edge_size as i32 + dx;
                let ny = mouse_y / self.pixel_edge_size as i32 + dy;

                if nx >= 0 && ny >= 0 && (nx as u32) < self.a_width && (ny as u32) < self.a_height {
                    let distance = ((dx * dx + dy * dy) as f64).sqrt();
                    if distance <= brush_radius as f64 {
                        let index = (ny as usize * self.a_width as usize + nx as usize) as usize;
                        self.pxl_vec[index] = state;
                    }
                }
            }
        }
    }

    pub fn resize(&mut self, new_width: u32, new_height: u32) {
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

    pub fn change_pixel_size(&mut self, delta: i32) {
        let new_pixel_size = (self.pixel_edge_size as i32 + delta).clamp(1, 50) as u32;
        if new_pixel_size != self.pixel_edge_size {
            self.pixel_edge_size = new_pixel_size;
            self.resize(self.width, self.height);
        }
    }

    pub fn change_parameter(&mut self, param: &str, delta: f64) {
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
            "noise_intensity" => {
                self.noise_intensity = (self.noise_intensity + delta).clamp(0.0, 1.0);
            }
            "info_bar_height" => {
                self.info_bar_height = (self.info_bar_height as f64 + delta).clamp(0.0, 200.0) as u32;
            }
            _ => {}
        }
    }

    pub fn reset_parameters(&mut self) {
        self.pixel_edge_size = DEFAULT_PIXEL_EDGE_SIZE;
        self.update_freq = DEFAULT_UPDATE_FREQ;
        self.kernel_rad = DEFAULT_KERNEL_RAD;
        self.bell_m = DEFAULT_BELL_M;
        self.bell_s = DEFAULT_BELL_S;
        self.info_bar_height = DEFAULT_INFO_BAR_HEIGHT;
        self.noise_intensity = DEFAULT_NOISE_INTENSITY;
        self.noise_enabled = true;
        self.resize(self.width, self.height);
    }

    pub fn switch_gradient(&mut self) {
        self.gradient_idx = (self.gradient_idx + 1) % self.gradients.len();
        self.colors = Self::compute_colors(self.gradients[self.gradient_idx].as_ref());
    }

    pub fn toggle_info_window(&mut self, video_subsystem: &sdl2::VideoSubsystem) {
        if let Some(_info_window) = &self.info_window {
            self.info_window = None;
        } else {
            let info_window = video_subsystem
                .window("Simulation Info", 500, 400)
                .position_centered()
                .build()
                .unwrap();

            let info_canvas = info_window.into_canvas().build().unwrap();
            self.info_window = Some(info_canvas.into_window());
        }
    }

    pub fn toggle_noise(&mut self) {
        self.noise_enabled = !self.noise_enabled;
    }
}

