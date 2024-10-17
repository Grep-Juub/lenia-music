use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::EventPump;
use sdl2::VideoSubsystem;
use crate::game::GameOfLife;

pub fn handle_events(event_pump: &mut EventPump, game: &mut GameOfLife, video_subsystem: &VideoSubsystem) -> bool {
    for event in event_pump.poll_iter() {
        game.handle_slider_events(&event);
        match event {
            Event::Quit { .. } |
            Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                return false;
            },
            Event::KeyDown { keycode: Some(Keycode::Space), .. } => {
                game.running = !game.running;
            },
            Event::KeyDown { keycode: Some(Keycode::H), .. } => {
                game.toggle_info_window(video_subsystem);
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
            Event::KeyDown { keycode: Some(Keycode::N), .. } => {
                game.toggle_noise();
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
                game.change_parameter("noise_intensity", 0.01);
            },
            Event::KeyDown { keycode: Some(Keycode::F10), .. } => {
                game.change_parameter("noise_intensity", -0.01);
            },
            Event::MouseButtonDown { mouse_btn, x, y, .. } => {
                game.mouse_down = true;
                let state = match mouse_btn {
                    sdl2::mouse::MouseButton::Left => 1.0,
                    sdl2::mouse::MouseButton::Right => 0.0,
                    _ => continue,
                };
                game.add_cells_with_brush(x, y, 5, state);
            },
            Event::MouseButtonUp { mouse_btn: sdl2::mouse::MouseButton::Left, .. }
            | Event::MouseButtonUp { mouse_btn: sdl2::mouse::MouseButton::Right, .. } => {
                game.mouse_down = false;
            },
            Event::MouseMotion { x, y, .. } => {
                if game.mouse_down {
                    game.add_cells_with_brush(x, y, 5, 1.0);
                }
            },
            Event::Window { win_event: sdl2::event::WindowEvent::Resized(new_width, new_height), .. } => {
                game.resize(new_width as u32, new_height as u32);
            },
            _ => {}
        }
    }
    true
}

