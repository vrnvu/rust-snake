use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute, terminal,
};
use rust_snake::{
    game::{Direction, Game},
    menu,
};
use std::{
    thread,
    time::{Duration, Instant},
};

const GAME_WIDTH: u16 = 30;
const PANEL_WIDTH: u16 = 20;
const HEIGHT: u16 = 15;
const FRAME_DURATION: Duration = Duration::from_millis(125); // 8 FPS

fn main() -> std::io::Result<()> {
    match menu::show(GAME_WIDTH, PANEL_WIDTH, HEIGHT)? {
        Some(player_name) => run_game(player_name),
        None => Ok(()),
    }
}

fn run_game(player_name: String) -> std::io::Result<()> {
    terminal::enable_raw_mode()?;
    execute!(std::io::stdout(), terminal::Clear(terminal::ClearType::All))?;
    execute!(std::io::stdout(), cursor::Hide)?;

    let mut game = Game::new(GAME_WIDTH, PANEL_WIDTH, HEIGHT, player_name);
    game.render()?;

    'game_loop: loop {
        let frame_start = Instant::now();

        let new_direction = event::poll(Duration::from_millis(10))?
            .then(|| event::read())
            .and_then(|result| result.ok())
            .and_then(|event| match event {
                Event::Key(key_event) => Some(key_event.code),
                _ => None,
            });

        if let Some(KeyCode::Esc) = new_direction {
            break 'game_loop;
        }

        let new_direction = new_direction.and_then(|code| match code {
            KeyCode::Up => Some(Direction::Up),
            KeyCode::Down => Some(Direction::Down),
            KeyCode::Left => Some(Direction::Left),
            KeyCode::Right => Some(Direction::Right),
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
    execute!(std::io::stdout(), cursor::Show)?;
    Ok(())
}
