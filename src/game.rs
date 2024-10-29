use crossterm::{
    cursor,
    event::KeyCode,
    queue,
    style::{self, Stylize},
};
use rand::Rng;
use std::{
    collections::VecDeque,
    io::{self},
};

use crate::theme;

pub struct GameState {
    pub snake: Snake,
    pub food: Food,
    pub score: u32,
    pub game_width: u16,
    pub game_height: u16,
    pub actions: Vec<Action>,
}

impl GameState {
    pub fn new(game_width: u16, game_height: u16) -> Self {
        let snake = Snake::new(game_width / 2, game_height / 2);
        let food = Food::new(game_width, game_height);
        let score = 0;

        Self {
            snake,
            food,
            score,
            game_width,
            game_height,
            actions: Vec::new(),
        }
    }

    pub fn queue(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        self.food.queue(stdout)?;
        self.snake.queue(stdout)?;
        Ok(())
    }

    pub fn next(&mut self, action: Action) {
        self.actions.push(action);

        if let Some(new_direction) = action.change_direction {
            self.snake.direction = new_direction;
        }

        if action.must_grow {
            self.snake.move_and_grow();
            self.food = Food::new(self.game_width, self.game_height);
            self.score += 1;
        }

        self.snake.move_direction();
    }

    pub fn is_game_over(&self) -> bool {
        self.snake
            .head
            .is_on_border(self.game_width, self.game_height)
            || self.snake.self_collision()
    }

    pub fn get_action(&self, user_input: Option<KeyCode>) -> Action {
        let direction = user_input.and_then(|code| match code {
            KeyCode::Up => Some(Direction::Up),
            KeyCode::Down => Some(Direction::Down),
            KeyCode::Left => Some(Direction::Left),
            KeyCode::Right => Some(Direction::Right),
            _ => None,
        });

        let must_grow = self.snake.head == self.food.position;

        if direction.is_none() {
            return Action::new(self.snake.head, None, must_grow);
        }

        let new_direction = direction.unwrap();
        if new_direction != self.snake.direction && new_direction != self.snake.direction.reverse()
        {
            return Action::new(self.snake.head, Some(new_direction), must_grow);
        }

        Action::new(self.snake.head, None, must_grow)
    }
}

pub struct GameGrid {
    pub width: u16,
    pub height: u16,
}

impl GameGrid {
    pub fn new(width: u16, height: u16) -> Self {
        Self { width, height }
    }

    pub fn queue(&self, stdout: &mut io::Stdout) -> io::Result<()> {
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn reverse(&self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
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

    pub fn move_direction(&self, direction: Direction) -> Position {
        let mut next = *self;
        match direction {
            Direction::Up => next.y -= 1,
            Direction::Down => next.y += 1,
            Direction::Left => next.x -= 1,
            Direction::Right => next.x += 1,
        }
        next
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
            head: Position::new(initial_x, initial_y),
            tail: VecDeque::new(),
            direction: Direction::Right,
            grow: false,
        }
    }

    pub fn queue(&self, stdout: &mut io::Stdout) -> io::Result<()> {
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
        let old_head = self.head;
        self.head = self.head.move_direction(self.direction);

        if !self.tail.is_empty() {
            self.tail.push_front(old_head);
            self.tail.pop_back();
        }
    }

    pub fn move_and_grow(&mut self) {
        let old_head = self.head;
        self.head = self.head.move_direction(self.direction);

        self.tail.push_front(old_head);
    }

    pub fn self_collision(&self) -> bool {
        self.tail
            .iter()
            .any(|pos| pos.x == self.head.x && pos.y == self.head.y)
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

    pub fn queue(&self, stdout: &mut io::Stdout) -> io::Result<()> {
        queue!(
            stdout,
            cursor::MoveTo(self.position.x, self.position.y),
            style::PrintStyledContent("●".with(theme::ACCENT).on(theme::BACKGROUND))
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Action {
    pub snake_head: Position,
    pub change_direction: Option<Direction>,
    pub must_grow: bool,
    pub food_position: Position,
    pub is_reverse: bool,
}

impl Action {
    pub fn new(snake_head: Position, change_direction: Option<Direction>, must_grow: bool) -> Self {
        Self {
            snake_head,
            change_direction,
            must_grow,
            food_position: Position::new(0, 0),
            is_reverse: false,
        }
    }

    pub fn reverse(action: Action) -> Self {
        let reverse_direction = action.change_direction.map(|d| d.reverse());
        Self {
            snake_head: action.snake_head,
            change_direction: reverse_direction,
            must_grow: !action.must_grow,
            food_position: action.food_position,
            is_reverse: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_snake_movement() {
        let mut snake = Snake::new(5, 5);
        assert_eq!(snake.head, Position::new(5, 5));
        assert!(snake.tail.is_empty());

        snake.direction = Direction::Right;
        snake.move_direction();
        assert_eq!(snake.head, Position::new(6, 5));
        assert!(snake.tail.is_empty());

        snake.direction = Direction::Down;
        snake.move_direction();
        assert_eq!(snake.head, Position::new(6, 6));
        assert!(snake.tail.is_empty());
    }

    #[test]
    fn test_snake_with_tail_size_one() {
        let mut snake = Snake::new(5, 5);
        snake.tail.push_back(Position::new(4, 5));

        snake.direction = Direction::Right;
        snake.move_direction();

        assert_eq!(snake.head, Position::new(6, 5));
        assert_eq!(snake.tail.len(), 1);
        assert_eq!(snake.tail.front().unwrap(), &Position::new(5, 5));

        snake.direction = Direction::Right;
        snake.move_direction();

        assert_eq!(snake.head, Position::new(7, 5));
        assert_eq!(snake.tail.len(), 1);
        assert_eq!(snake.tail.front().unwrap(), &Position::new(6, 5));

        snake.direction = Direction::Up;
        snake.move_direction();

        assert_eq!(snake.head, Position::new(7, 4));
        assert_eq!(snake.tail.len(), 1);
        assert_eq!(snake.tail.front().unwrap(), &Position::new(7, 5));

        snake.direction = Direction::Up;
        snake.move_direction();

        assert_eq!(snake.head, Position::new(7, 3));
        assert_eq!(snake.tail.len(), 1);
        assert_eq!(snake.tail.front().unwrap(), &Position::new(7, 4));
    }

    #[test]
    fn test_snake_with_tail_size_two() {
        let mut snake = Snake::new(5, 5);
        snake.tail.push_back(Position::new(4, 5));
        snake.tail.push_back(Position::new(3, 5));

        snake.direction = Direction::Right;
        snake.move_direction();

        assert_eq!(snake.head, Position::new(6, 5));
        assert_eq!(snake.tail.len(), 2);
        assert_eq!(snake.tail.front().unwrap(), &Position::new(5, 5));
        assert_eq!(snake.tail.back().unwrap(), &Position::new(4, 5));

        snake.direction = Direction::Right;
        snake.move_direction();

        assert_eq!(snake.head, Position::new(7, 5));
        assert_eq!(snake.tail.len(), 2);
        assert_eq!(snake.tail.front().unwrap(), &Position::new(6, 5));
        assert_eq!(snake.tail.back().unwrap(), &Position::new(5, 5));

        snake.direction = Direction::Up;
        snake.move_direction();

        assert_eq!(snake.head, Position::new(7, 4));
        assert_eq!(snake.tail.len(), 2);
        assert_eq!(snake.tail.front().unwrap(), &Position::new(7, 5));
        assert_eq!(snake.tail.back().unwrap(), &Position::new(6, 5));
    }

    #[test]
    fn test_snake_self_collision() {
        let mut snake = Snake::new(5, 5);
        snake.tail.push_back(Position::new(4, 5));
        assert!(!snake.self_collision());

        snake.tail.push_back(Position::new(5, 5));
        assert!(snake.self_collision());
    }
}
