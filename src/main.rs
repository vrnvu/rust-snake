use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute, queue,
    style::{self, Stylize},
    terminal,
};
use rand::Rng;
use rust_snake::game::{Direction, Game};
use std::{
    collections::VecDeque,
    io::{self, Write},
    thread,
    time::{Duration, Instant},
};

fn main() -> io::Result<()> {
    let width = 30;
    let height = 15;
    const FRAME_DURATION: Duration = Duration::from_millis(125); // 8 FPS

    terminal::enable_raw_mode()?;
    let mut game = Game::new(width, height);
    game.init()?;
    game.render()?;

    loop {
        let frame_start = Instant::now();

        // Handle input (with shorter poll time to remain responsive)
        let new_direction = event::poll(Duration::from_millis(10))?
            .then(|| event::read())
            .and_then(|result| result.ok())
            .and_then(|event| match event {
                Event::Key(key_event) => Some(key_event.code),
                _ => None,
            })
            .and_then(|code| match code {
                KeyCode::Up => Some(Direction::Up),
                KeyCode::Down => Some(Direction::Down),
                KeyCode::Left => Some(Direction::Left),
                KeyCode::Right => Some(Direction::Right),
                KeyCode::Esc => panic!("Game ended"), // TODO handle break differently
                _ => None,
            });

        game.update(new_direction)?;
        game.render()?;

        if game.is_game_over() {
            break;
        }

        // Calculate remaining time in frame and sleep
        let elapsed = frame_start.elapsed();
        if elapsed < FRAME_DURATION {
            thread::sleep(FRAME_DURATION - elapsed);
        }
    }

    // Cleanup
    terminal::disable_raw_mode()?;
    execute!(io::stdout(), cursor::Show)?;
    Ok(())
}
