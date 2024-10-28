use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute, terminal,
};
use rust_snake::{
    game::{Direction, Food, GameFrame, SidePanel, Snake},
    menu,
};
use std::{
    io::Write,
    thread,
    time::{Duration, Instant},
};

const GAME_WIDTH: u16 = 30;
const PANEL_WIDTH: u16 = 20;
const HEIGHT: u16 = 15;
const FRAME_DURATION: Duration = Duration::from_millis(125); // 8 FPS

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

    let game_frame = GameFrame::new(GAME_WIDTH, HEIGHT);
    let mut snake = Snake::new(GAME_WIDTH / 2, HEIGHT / 2);
    let mut food = Food::new(GAME_WIDTH, HEIGHT);
    let mut side_panel = SidePanel::new(GAME_WIDTH, HEIGHT, PANEL_WIDTH, player_name);
    let mut score = 0;

    game_frame.render(stdout)?;

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

        if let Some(direction) = new_direction {
            snake.direction = direction;
        }

        if snake.head.is_on(&food.position) {
            snake.grow = true;
            score += 1;
            side_panel.update_score(score);
            food = Food::new(game_frame.width, game_frame.height);
        }

        snake.move_direction();

        game_frame.render(stdout)?;
        food.render(stdout)?;
        snake.render(stdout)?;
        side_panel.render(stdout)?;

        stdout.flush()?;

        if snake.head.is_on_border(game_frame.width, game_frame.height)
            || snake.head.self_collision(&snake.tail)
        {
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
