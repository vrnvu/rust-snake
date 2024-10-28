use crossterm::{
    cursor, queue,
    style::{self, Stylize},
};
use rand::Rng;
use std::{
    collections::VecDeque,
    io::{self},
};

use crate::theme;

pub struct GameFrame {
    pub width: u16,
    pub height: u16,
}

impl GameFrame {
    pub fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }

    pub fn render(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        for y in 0..self.height {
            for x in 0..self.width {
                queue!(stdout, cursor::MoveTo(x, y))?;
                if Position::new(x, y).is_on_border(self.width, self.height) {
                    queue!(stdout, style::PrintStyledContent("█".with(theme::SURFACE)))?;
                    continue;
                }
                queue!(
                    stdout,
                    style::PrintStyledContent("█".with(theme::BACKGROUND))
                )?;
            }
        }
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
pub struct Position {
    pub x: u16,
    pub y: u16,
}

impl Position {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }

    pub fn is_on_border(&self, width: u16, height: u16) -> bool {
        self.x == 0 || self.y == height - 1 || self.x == width - 1 || self.y == 0
    }

    pub fn is_on(&self, other: &Position) -> bool {
        self.x == other.x && self.y == other.y
    }

    pub fn self_collision(&self, tail: &VecDeque<Position>) -> bool {
        tail.iter().any(|pos| pos.x == self.x && pos.y == self.y)
    }
}

#[derive(Debug)]
pub struct Snake {
    pub head: Position,
    pub tail: VecDeque<Position>,
    pub direction: Direction,
    pub grow: bool,
}

impl Snake {
    pub fn new(initial_x: u16, initial_y: u16) -> Self {
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

    pub fn render(&self, stdout: &mut io::Stdout) -> io::Result<()> {
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

    pub fn move_direction(&mut self) {
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
pub struct Food {
    pub position: Position,
}

impl Food {
    pub fn new(max_width: u16, max_height: u16) -> Self {
        let mut rng = rand::thread_rng();
        let position = Position::new(
            rng.gen_range(1..max_width - 1),
            rng.gen_range(1..max_height - 1),
        );
        Self { position }
    }

    pub fn render(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        queue!(
            stdout,
            cursor::MoveTo(self.position.x, self.position.y),
            style::PrintStyledContent("●".with(theme::ACCENT).on(theme::BACKGROUND))
        )?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct InfoRow {
    pub title: String,
    pub data: String,
    pub y_position: u16,
}

impl InfoRow {
    pub fn new(title: &str, data: &str, row_index: u16) -> Self {
        Self {
            title: title.to_string(),
            data: data.to_string(),
            y_position: row_index * 3, // Each row takes 2 lines + 1 space
        }
    }

    pub fn render(&self, stdout: &mut io::Stdout, x_offset: u16) -> io::Result<()> {
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
pub struct SidePanel {
    pub x: u16,
    pub width: u16,
    pub height: u16,
    pub player_row: InfoRow,
    pub score_row: InfoRow,
    pub max_score_row: InfoRow,
}

impl SidePanel {
    pub fn new(game_width_offset: u16, height: u16, panel_width: u16, player_name: String) -> Self {
        Self {
            x: game_width_offset + 2,
            width: panel_width,
            height,
            player_row: InfoRow::new("PLAYER", &player_name, 0),
            score_row: InfoRow::new("SCORE", "0", 1),
            max_score_row: InfoRow::new("MAX SCORE", "25", 2), // TODO
        }
    }

    pub fn render(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        self.render_borders_and_corners(stdout)?;

        self.player_row.render(stdout, self.x)?;
        self.score_row.render(stdout, self.x)?;
        self.max_score_row.render(stdout, self.x)?;
        Ok(())
    }

    pub fn update_score(&mut self, score: u32) {
        self.score_row.data = score.to_string();
    }

    pub fn render_borders_and_corners(&self, stdout: &mut io::Stdout) -> io::Result<()> {
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
