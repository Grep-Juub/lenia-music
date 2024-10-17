use sdl2::event::Event;
use sdl2::mouse::MouseButton;
use sdl2::keyboard::Keycode;
use sdl2::EventPump;
use sdl2::VideoSubsystem;
use crate::game::GameOfLife;

pub fn handle_events(event_pump: &mut EventPump, game: &mut GameOfLife, video_subsystem: &VideoSubsystem) -> bool {
    for event in event_pump.poll_iter() {
        match event {
            // Quit event handling
            Event::Quit { .. } |
            Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                return false;
            },

            // Pause/Resume the game with Space key
            Event::KeyDown { keycode: Some(Keycode::Space), .. } => {
                game.running = !game.running;
            },

            // Handle window resizing
            Event::Window { win_event: sdl2::event::WindowEvent::Resized(new_width, new_height), .. } => {
                game.resize(new_width as u32, new_height as u32);
            },

            // Mouse button down: Start adding live or dead cells
            Event::MouseButtonDown { mouse_btn, x, y, .. } => {
                game.mouse_down = true;
                let state = match mouse_btn {
                    MouseButton::Left => 1.0,  // Add live cell
                    MouseButton::Right => 0.0, // Add dead cell
                    _ => continue,
                };
                game.add_cells_with_brush(x, y, 5, state);
            },

            // Mouse button up: Stop adding cells
            Event::MouseButtonUp { mouse_btn: MouseButton::Left, .. }
            | Event::MouseButtonUp { mouse_btn: MouseButton::Right, .. } => {
                game.mouse_down = false;
            },

            // Mouse motion: Add cells continuously while the mouse is held down
            Event::MouseMotion { x, y, .. } => {
                if game.mouse_down {
                    game.add_cells_with_brush(x, y, 5, 1.0);
                }
            },

            _ => {}
        }
    }
    true
}

