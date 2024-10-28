use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute, terminal,
};
use rust_snake::{
    game::{Direction, GameGrid, GameState},
    menu,
    menu::SidePanel,
};
use std::{
    io::Write,
    thread,
    time::{Duration, Instant},
};

const GAME_WIDTH: u16 = 30;
const PANEL_WIDTH: u16 = 20;
const HEIGHT: u16 = 15;
const FRAME_DURATION: Duration = Duration::from_millis(75); // ~13 FPS

fn main() -> std::io::Result<()> {
    let mut stdout = std::io::stdout();
    if let Some(player_name) = menu::show(&mut stdout, GAME_WIDTH, PANEL_WIDTH, HEIGHT)? {
        run_game(&mut stdout, player_name)?;
    }

    execute!(
        stdout,
        terminal::Clear(terminal::ClearType::All),
        cursor::MoveTo(0, 0),
        cursor::Show
    )?;
    terminal::disable_raw_mode()?;
    Ok(())
}

fn run_game(stdout: &mut std::io::Stdout, player_name: String) -> std::io::Result<()> {
    terminal::enable_raw_mode()?;
    execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
    execute!(stdout, cursor::Hide)?;

    let game_grid = GameGrid::new(GAME_WIDTH, HEIGHT);
    let mut state = GameState::new(GAME_WIDTH, HEIGHT);
    let mut side_panel = SidePanel::new(GAME_WIDTH, HEIGHT, PANEL_WIDTH, player_name);

    'game_loop: loop {
        let frame_start = Instant::now();

        let user_input = event::poll(Duration::from_millis(5))?
            .then(event::read)
            .and_then(|result| result.ok())
            .and_then(|event| match event {
                Event::Key(key_event) => Some(key_event.code),
                _ => None,
            });

        if let Some(KeyCode::Esc) = user_input {
            break 'game_loop;
        }

        if let Some(KeyCode::Char('s')) = user_input {
            loop {
                let user_input = event::poll(Duration::from_millis(5))?
                    .then(event::read)
                    .and_then(|result| result.ok())
                    .and_then(|event| match event {
                        Event::Key(key_event) => Some(key_event.code),
                        _ => None,
                    });

                if let Some(KeyCode::Esc) = user_input {
                    break 'game_loop;
                }

                if let Some(KeyCode::Char('s')) = user_input {
                    break;
                }

                if let Some(KeyCode::Char('b')) = user_input {
                    // TODO
                    println!("magic!");
                }
            }
        }

        let game_action = user_input.and_then(|code| match code {
            KeyCode::Up => Some(Direction::Up),
            KeyCode::Down => Some(Direction::Down),
            KeyCode::Left => Some(Direction::Left),
            KeyCode::Right => Some(Direction::Right),
            _ => None,
        });

        state.next(game_action);
        side_panel.update_score(state.score);

        game_grid.queue(stdout)?;
        side_panel.queue(stdout)?;
        state.queue(stdout)?;
        stdout.flush()?;

        if state.is_game_over() {
            break 'game_loop;
        }

        // Calculate remaining time in frame and sleep
        let elapsed = frame_start.elapsed();
        if elapsed < FRAME_DURATION {
            thread::sleep(FRAME_DURATION - elapsed);
        }
    }

    Ok(())
}
