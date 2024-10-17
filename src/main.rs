mod game;
mod render;
mod ui;
mod utils;

use sdl2::Sdl;
use rayon::ThreadPoolBuilder;

use game::GameOfLife;
use ui::handle_events;

fn main() {
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

    let mut game = GameOfLife::new(750, 750, game::DEFAULT_PIXEL_EDGE_SIZE, &video_subsystem);

    'running: loop {
        if !handle_events(&mut event_pump, &mut game, &video_subsystem) {
            break 'running;
        }

        game.update();
        game.draw(&mut canvas);
        game.update_info_window(&font);
    }
}
