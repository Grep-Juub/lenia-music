mod game;
mod render;
mod ui;
mod utils;

use sdl2::Sdl;
use rayon::ThreadPoolBuilder;
use imgui::*;
use imgui_sdl2::ImguiSdl2;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::video::Window;
use sdl2::render::Canvas;
use sdl2::pixels::Color;
use sdl2::video::GLProfile;
use std::time::Instant;

use game::GameOfLife;
use ui::handle_events;

fn main() {
    // Set up SDL2 and ImGui context
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(GLProfile::Core);
    gl_attr.set_context_version(4, 1);

    let window = video_subsystem
        .window("Game of Life", 750, 750)
        .position_centered()
        .resizable()
        .opengl()
        .build()
        .unwrap();

    let _gl_context = window.gl_create_context().unwrap();
    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut game = GameOfLife::new(750, 750, game::DEFAULT_PIXEL_EDGE_SIZE, &video_subsystem);

    // Initialize ImGui context
    let mut imgui = Context::create();
    let mut imgui_sdl2 = ImguiSdl2::new(&mut imgui, &window);
    let mut last_frame = Instant::now();

    'running: loop {
        // Handle SDL2 events
        for event in event_pump.poll_iter() {
            if let Event::Quit { .. } = event {
                break 'running;
            }
            imgui_sdl2.handle_event(&mut imgui, &event);
        }

        let now = Instant::now();
        let delta = now - last_frame;
        let delta_s = delta.as_secs_f32();
        last_frame = now;

        let ui = imgui.frame();
        imgui_sdl2.prepare_frame(ui.io_mut(), &window, &event_pump.mouse_state());

        // Render ImGui UI
        Window::new("Game Parameters").build(&ui, || {
            ui.text("Adjust the Game of Life parameters:");
            Slider::new("Update Frequency", 1.0, 100.0).build(&ui, &mut game.update_freq);
            Slider::new("Kernel Radius", 1.0, 20.0).build(&ui, &mut game.kernel_rad);
            Slider::new("Bell M", 0.01, 1.0).build(&ui, &mut game.bell_m);
            Slider::new("Bell S", 0.01, 1.0).build(&ui, &mut game.bell_s);
            Slider::new("Noise Intensity", 0.0, 1.0).build(&ui, &mut game.noise_intensity);
        });

        // Clear the SDL2 canvas
        canvas.set_draw_color(Color::RGB(10, 20, 30));
        canvas.clear();

        // Render the game
        game.update();
        game.draw(&mut canvas);

        // Render ImGui on top of the game
        window.gl_make_current(&_gl_context).unwrap();
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        imgui_sdl2.prepare_render(&ui, &window);
        imgui.render();
        canvas.present();
    }
}

