use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute, queue,
    style::{self, Stylize},
    terminal,
};
use rand::Rng;
use std::{
    collections::VecDeque,
    io::{self, Write},
    thread,
    time::{Duration, Instant},
};

#[derive(Debug)]
struct Game {
    stdout: io::Stdout,
    width: u16,
    height: u16,
    snake: Snake,
    food: Food,
}

impl Game {
    fn new(width: u16, height: u16) -> Self {
        Self {
            stdout: io::stdout(),
            width,
            height,
            snake: Snake::new(width / 2, height / 2),
            food: Food::new(width, height),
        }
    }

    fn init(&mut self) -> io::Result<()> {
        execute!(self.stdout, terminal::Clear(terminal::ClearType::All))?;
        execute!(self.stdout, cursor::Hide)?;
        Ok(())
    }

    fn render(&mut self) -> io::Result<()> {
        for y in 0..self.height {
            for x in 0..self.width {
                queue!(self.stdout, cursor::MoveTo(x, y))?;
                if Position::new(x, y).is_on_border(self.width, self.height) {
                    queue!(self.stdout, style::PrintStyledContent("█".white()))?;
                    continue;
                }

                queue!(self.stdout, style::PrintStyledContent("█".dark_blue()))?;
            }
        }

        self.food.render(&mut self.stdout)?;
        self.snake.render(&mut self.stdout)?;
        self.stdout.flush()
    }

    fn is_game_over(&self) -> bool {
        self.snake.head.is_on_border(self.width, self.height)
            || self.snake.head.self_collision(&self.snake.tail)
    }

    // TODO: make this functional and reproducible
    // Old State -> Action -> New State
    fn update(&mut self) -> io::Result<()> {
        if self.snake.head.is_on(&self.food.position) {
            self.snake.grow = true;
            self.food = Food::new(self.width, self.height);
        }

        self.snake.move_direction();
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug, Clone)]
struct Position {
    x: u16,
    y: u16,
}

impl Position {
    fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }

    fn is_on_border(&self, width: u16, height: u16) -> bool {
        self.x == 0 || self.y == height - 1 || self.x == width - 1 || self.y == 0
    }

    fn is_on(&self, other: &Position) -> bool {
        self.x == other.x && self.y == other.y
    }

    fn self_collision(&self, tail: &VecDeque<Position>) -> bool {
        tail.iter().any(|pos| pos.x == self.x && pos.y == self.y)
    }
}

#[derive(Debug)]
struct Snake {
    head: Position,
    tail: VecDeque<Position>,
    direction: Direction,
    grow: bool,
}

impl Snake {
    fn new(initial_x: u16, initial_y: u16) -> Self {
        Self {
            head: Position {
                x: initial_x,
                y: initial_y,
            },
            tail: VecDeque::new(),
            direction: Direction::Right,
            grow: false,
        }
    }

    fn render(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        for pos in &self.tail {
            queue!(
                stdout,
                cursor::MoveTo(pos.x, pos.y),
                style::PrintStyledContent("█".green())
            )?;
        }

        queue!(
            stdout,
            cursor::MoveTo(self.head.x, self.head.y),
            style::PrintStyledContent("█".yellow())
        )?;

        Ok(())
    }

    fn move_direction(&mut self) {
        let old_head = self.head.clone();

        match self.direction {
            Direction::Up => self.head.y -= 1,
            Direction::Down => self.head.y += 1,
            Direction::Left => self.head.x -= 1,
            Direction::Right => self.head.x += 1,
        }

        self.tail.push_front(old_head);

        // TODO should be functional, probably no need for a state if we have actions
        if !self.grow {
            self.tail.pop_back();
        }
        self.grow = false;
    }
}

#[derive(Debug)]
struct Food {
    position: Position,
}

impl Food {
    fn new(max_width: u16, max_height: u16) -> Self {
        let mut rng = rand::thread_rng();
        let position = Position::new(
            rng.gen_range(1..max_width - 1),
            rng.gen_range(1..max_height - 1),
        );
        Self { position }
    }

    fn render(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        queue!(
            stdout,
            cursor::MoveTo(self.position.x, self.position.y),
            style::PrintStyledContent("●".red())
        )?;
        Ok(())
    }
}

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
        if event::poll(Duration::from_millis(10))? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Up => game.snake.direction = Direction::Up,
                    KeyCode::Down => game.snake.direction = Direction::Down,
                    KeyCode::Left => game.snake.direction = Direction::Left,
                    KeyCode::Right => game.snake.direction = Direction::Right,
                    KeyCode::Esc => break,
                    _ => {}
                }
            }
        }

        game.update()?;
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
