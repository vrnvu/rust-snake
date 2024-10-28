use crossterm::{
    cursor, queue,
    style::{self, Stylize},
};
use rand::Rng;
use std::{
    collections::VecDeque,
    io::{self, Write},
};

use crate::theme;

#[derive(Debug)]
pub struct Game {
    stdout: io::Stdout,
    width: u16,
    height: u16,
    snake: Snake,
    food: Food,
    side_panel: SidePanel,
    score: u32,
}

impl Game {
    pub fn new(width: u16, panel_width: u16, height: u16, player_name: String) -> Self {
        Self {
            stdout: io::stdout(),
            width,
            height,
            snake: Snake::new(width / 2, height / 2),
            food: Food::new(width, height),
            side_panel: SidePanel::new(width, height, panel_width, player_name),
            score: 0,
        }
    }

    pub fn render(&mut self) -> io::Result<()> {
        for y in 0..self.height {
            for x in 0..self.width {
                queue!(self.stdout, cursor::MoveTo(x, y))?;
                if Position::new(x, y).is_on_border(self.width, self.height) {
                    queue!(
                        self.stdout,
                        style::PrintStyledContent("█".with(theme::SURFACE))
                    )?;
                    continue;
                }
                queue!(
                    self.stdout,
                    style::PrintStyledContent("█".with(theme::BACKGROUND))
                )?;
            }
        }

        self.food.render(&mut self.stdout)?;
        self.snake.render(&mut self.stdout)?;
        self.side_panel.render(&mut self.stdout)?;

        self.stdout.flush()
    }

    pub fn is_game_over(&self) -> bool {
        self.snake.head.is_on_border(self.width, self.height)
            || self.snake.head.self_collision(&self.snake.tail)
    }

    pub fn update(&mut self, new_direction: Option<Direction>) -> io::Result<()> {
        if let Some(direction) = new_direction {
            self.snake.direction = direction;
        }

        if self.snake.head.is_on(&self.food.position) {
            self.snake.grow = true;
            self.score += 1;
            self.side_panel.update_score(self.score);
            self.food = Food::new(self.width, self.height);
        }

        self.snake.move_direction();
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
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
                style::PrintStyledContent("█".with(theme::SECONDARY))
            )?;
        }

        queue!(
            stdout,
            cursor::MoveTo(self.head.x, self.head.y),
            style::PrintStyledContent("█".with(theme::PRIMARY))
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
            style::PrintStyledContent("●".with(theme::ACCENT).on(theme::BACKGROUND))
        )?;
        Ok(())
    }
}

#[derive(Debug)]
struct InfoRow {
    title: String,
    data: String,
    y_position: u16,
}

impl InfoRow {
    fn new(title: &str, data: &str, row_index: u16) -> Self {
        Self {
            title: title.to_string(),
            data: data.to_string(),
            y_position: row_index * 3, // Each row takes 2 lines + 1 space
        }
    }

    fn render(&self, stdout: &mut io::Stdout, x_offset: u16) -> io::Result<()> {
        queue!(
            stdout,
            cursor::MoveTo(x_offset + 2, self.y_position),
            style::PrintStyledContent(self.title.as_str().white())
        )?;
        queue!(
            stdout,
            cursor::MoveTo(x_offset + 2, self.y_position + 1),
            style::PrintStyledContent(self.data.as_str().white())
        )?;
        Ok(())
    }
}

#[derive(Debug)]
struct SidePanel {
    x: u16,
    width: u16,
    height: u16,
    player_row: InfoRow,
    score_row: InfoRow,
    max_score_row: InfoRow,
}

impl SidePanel {
    fn new(game_width_offset: u16, height: u16, panel_width: u16, player_name: String) -> Self {
        Self {
            x: game_width_offset + 2,
            width: panel_width,
            height,
            player_row: InfoRow::new("PLAYER", &player_name, 0),
            score_row: InfoRow::new("SCORE", "0", 1),
            max_score_row: InfoRow::new("MAX SCORE", "25", 2), // TODO
        }
    }

    fn render(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        self.render_borders_and_corners(stdout)?;

        self.player_row.render(stdout, self.x)?;
        self.score_row.render(stdout, self.x)?;
        self.max_score_row.render(stdout, self.x)?;
        Ok(())
    }

    fn update_score(&mut self, score: u32) {
        self.score_row.data = score.to_string();
    }

    fn render_borders_and_corners(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        // Draw vertical borders
        for y in 0..self.height {
            queue!(
                stdout,
                cursor::MoveTo(self.x, y),
                style::PrintStyledContent("│".with(theme::SURFACE))
            )?;
            queue!(
                stdout,
                cursor::MoveTo(self.x + self.width, y),
                style::PrintStyledContent("│".with(theme::SURFACE))
            )?;
        }

        // Draw horizontal borders
        for x in self.x..=self.x + self.width {
            queue!(
                stdout,
                cursor::MoveTo(x, 0),
                style::PrintStyledContent("─".with(theme::SURFACE))
            )?;
            queue!(
                stdout,
                cursor::MoveTo(x, self.height - 1),
                style::PrintStyledContent("─".with(theme::SURFACE))
            )?;
        }

        // Draw corners
        queue!(
            stdout,
            cursor::MoveTo(self.x, 0),
            style::PrintStyledContent("┌".with(theme::SURFACE))
        )?;
        queue!(
            stdout,
            cursor::MoveTo(self.x + self.width, 0),
            style::PrintStyledContent("┐".with(theme::SURFACE))
        )?;
        queue!(
            stdout,
            cursor::MoveTo(self.x, self.height - 1),
            style::PrintStyledContent("└".with(theme::SURFACE))
        )?;
        queue!(
            stdout,
            cursor::MoveTo(self.x + self.width, self.height - 1),
            style::PrintStyledContent("┘".with(theme::SURFACE))
        )?;

        Ok(())
    }
}
