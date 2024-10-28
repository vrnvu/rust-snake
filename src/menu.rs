use crate::theme;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute, queue,
    style::{self, Print, PrintStyledContent, Stylize},
    terminal,
};
use std::io::Write;

#[derive(Debug)]
pub struct SidePanel {
    pub x: u16,
    pub width: u16,
    pub height: u16,
    pub player_row: InfoRow<String>,
    pub score_row: InfoRow<u32>,
    pub max_score_row: InfoRow<u32>,
}

impl SidePanel {
    pub fn new(game_width_offset: u16, height: u16, panel_width: u16, player_name: String) -> Self {
        let x = game_width_offset + 2;
        Self {
            x,
            width: panel_width,
            height,
            player_row: InfoRow::new("PLAYER", player_name, x, 0),
            score_row: InfoRow::new("SCORE", 0, x, 1),
            max_score_row: InfoRow::new("MAX SCORE", 25, x, 2), // TODO
        }
    }

    pub fn queue(&self, stdout: &mut std::io::Stdout) -> std::io::Result<()> {
        self.queue_borders_and_corners(stdout)?;
        self.player_row.queue(stdout)?;
        self.score_row.queue(stdout)?;
        self.max_score_row.queue(stdout)?;

        // Add help text with some spacing after the info rows
        queue!(
            stdout,
            cursor::MoveTo(self.x + 2, self.max_score_row.y_position + 3),
            style::PrintStyledContent("CONTROLS".white()),
            cursor::MoveTo(self.x + 2, self.max_score_row.y_position + 4),
            style::PrintStyledContent("'s' to stop".white()),
            cursor::MoveTo(self.x + 2, self.max_score_row.y_position + 5),
            style::PrintStyledContent("'b' to go back".white()),
            cursor::MoveTo(self.x + 2, self.max_score_row.y_position + 6),
            style::PrintStyledContent("'ESC' to exit".white())
        )?;

        Ok(())
    }

    pub fn update_score(&mut self, score: u32) {
        self.score_row.data = score;
    }

    pub fn queue_borders_and_corners(&self, stdout: &mut std::io::Stdout) -> std::io::Result<()> {
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

#[derive(Debug)]
pub struct InfoRow<T: std::fmt::Display> {
    pub title: String,
    pub data: T,
    pub x_offset: u16,
    pub y_position: u16,
}

impl<T: std::fmt::Display> InfoRow<T> {
    pub fn new(title: &str, data: T, x_offset: u16, row_index: u16) -> Self {
        Self {
            title: title.to_string(),
            data,
            x_offset,
            y_position: row_index * 3, // Each row takes 2 lines + 1 space
        }
    }

    pub fn queue(&self, stdout: &mut std::io::Stdout) -> std::io::Result<()> {
        queue!(
            stdout,
            cursor::MoveTo(self.x_offset + 2, self.y_position),
            style::PrintStyledContent(self.title.as_str().white())
        )?;
        queue!(
            stdout,
            cursor::MoveTo(self.x_offset + 2, self.y_position + 1),
            style::PrintStyledContent(self.data.to_string().white())
        )?;
        Ok(())
    }
}

pub struct InputInfoRow {
    pub x: u16,
    pub y: u16,
    pub label: String,
    pub value: String,
    pub cursor_position: usize,
}

impl InputInfoRow {
    pub fn new(x: u16, y: u16, label: &str) -> Self {
        Self {
            x,
            y,
            label: label.to_string(),
            value: String::new(),
            cursor_position: 0,
        }
    }

    pub fn queue(&self, stdout: &mut std::io::Stdout) -> std::io::Result<()> {
        queue!(
            stdout,
            cursor::MoveTo(self.x, self.y),
            terminal::Clear(terminal::ClearType::CurrentLine), // Clear the line first
            Print(format!("{}: ", self.label)),
            Print(&self.value),
            cursor::MoveTo(
                self.x + self.label.len() as u16 + 2 + self.cursor_position as u16,
                self.y,
            ),
            Print("▎")
        )?;
        Ok(())
    }

    pub fn handle_input(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char(c) => {
                self.value.insert(self.cursor_position, c);
                self.cursor_position += 1;
            }
            KeyCode::Backspace if self.cursor_position > 0 => {
                // Fix: First store the target position
                let target_pos = self.cursor_position - 1;
                // Then remove the character at that position
                self.value.remove(target_pos);
                // Finally update cursor
                self.cursor_position = target_pos;
            }
            KeyCode::Left if self.cursor_position > 0 => {
                self.cursor_position -= 1;
            }
            KeyCode::Right if self.cursor_position < self.value.len() => {
                self.cursor_position += 1;
            }
            _ => {}
        }
    }
}

pub struct Button {
    pub x: u16,
    pub y: u16,
    pub label: String,
    pub selected: bool,
}

impl Button {
    pub fn new(x: u16, y: u16, label: &str, selected: bool) -> Self {
        Self {
            x,
            y,
            label: label.to_string(),
            selected,
        }
    }

    pub fn queue(&self, stdout: &mut std::io::Stdout) -> std::io::Result<()> {
        let border = "─".repeat(self.label.len() + 2);

        if self.selected {
            queue!(
                stdout,
                cursor::MoveTo(self.x - 2, self.y),
                Print(format!("> ┌{}┐", border)),
                cursor::MoveTo(self.x - 2, self.y + 1),
                Print(format!("  │ {} │", self.label)),
                cursor::MoveTo(self.x - 2, self.y + 2),
                Print(format!("  └{}┘ <", border))
            )?;
        } else {
            queue!(
                stdout,
                cursor::MoveTo(self.x - 2, self.y),
                Print(format!("  ┌{}┐", border)), // Added 2 spaces to align with selected state
                cursor::MoveTo(self.x - 2, self.y + 1),
                Print(format!("  │ {} │", self.label)),
                cursor::MoveTo(self.x - 2, self.y + 2),
                Print(format!("  └{}┘  ", border)) // Added 2 spaces to clear the '<'
            )?;
        }
        Ok(())
    }
}

pub fn show(
    stdout: &mut std::io::Stdout,
    game_width: u16,
    panel_width: u16,
    height: u16,
) -> std::io::Result<Option<String>> {
    let total_width = game_width + panel_width;
    terminal::enable_raw_mode()?;

    execute!(
        stdout,
        terminal::Clear(terminal::ClearType::All),
        cursor::Hide
    )?;

    // Draw complete frame
    for y in 0..height {
        for x in 0..total_width {
            queue!(stdout, cursor::MoveTo(x, y))?;
            if y == 0 || y == height - 1 || x == 0 || x == total_width - 1 {
                queue!(stdout, PrintStyledContent("█".with(theme::SURFACE)))?;
            } else {
                queue!(stdout, Print(" "))?;
            }
        }
    }

    let mut name_input = InputInfoRow::new(4, 2, "Your name");
    let center_x = total_width / 2;
    let mut play_button = Button::new(center_x - 10, height / 2, "PLAY", true);
    let mut exit_button = Button::new(center_x + 5, height / 2, "EXIT", false);
    let mut selected_button = 0;

    loop {
        name_input.queue(stdout)?;
        play_button.queue(stdout)?;
        exit_button.queue(stdout)?;

        // Help text aligned left
        queue!(
            stdout,
            cursor::MoveTo(4, height / 2 + 3),
            Print("Enter your name"),
            cursor::MoveTo(4, height / 2 + 4),
            Print("ENTER to select"),
            cursor::MoveTo(4, height / 2 + 5),
            Print("Press TAB to switch buttons"),
            cursor::MoveTo(4, height / 2 + 6),
            Print("ESC to exit")
        )?;

        stdout.flush()?;

        if let Event::Key(key_event) = event::read()? {
            match key_event.code {
                KeyCode::Esc => return Ok(None),
                KeyCode::Tab => {
                    selected_button = 1 - selected_button;
                    play_button.selected = selected_button == 0;
                    exit_button.selected = selected_button == 1;
                }
                KeyCode::Enter => {
                    return Ok(match selected_button {
                        0 => Some(name_input.value.clone()),
                        _ => None,
                    });
                }
                key => name_input.handle_input(key),
            }
        }
    }
}
