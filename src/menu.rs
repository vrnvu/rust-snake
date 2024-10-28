use crate::theme;
use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute, queue,
    style::{Print, PrintStyledContent, Stylize},
    terminal,
};
use std::io::{stdout, Write};

pub struct InputInfoRow {
    pub label: String,
    pub value: String,
    pub cursor_position: usize,
}

pub struct Button {
    pub label: String,
    pub selected: bool,
}

impl InputInfoRow {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            value: String::new(),
            cursor_position: 0,
        }
    }

    pub fn render(&self, x: u16, y: u16, stdout: &mut std::io::Stdout) -> std::io::Result<()> {
        queue!(
            stdout,
            cursor::MoveTo(x, y),
            terminal::Clear(terminal::ClearType::CurrentLine), // Clear the line first
            Print(format!("{}: ", self.label)),
            Print(&self.value),
            cursor::MoveTo(
                x + self.label.len() as u16 + 2 + self.cursor_position as u16,
                y,
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

impl Button {
    pub fn new(label: &str, selected: bool) -> Self {
        Self {
            label: label.to_string(),
            selected,
        }
    }

    pub fn render(&self, x: u16, y: u16, stdout: &mut std::io::Stdout) -> std::io::Result<()> {
        let border = "─".repeat(self.label.len() + 2);

        if self.selected {
            queue!(
                stdout,
                cursor::MoveTo(x - 2, y),
                Print(format!("> ┌{}┐", border)),
                cursor::MoveTo(x - 2, y + 1),
                Print(format!("  │ {} │", self.label)),
                cursor::MoveTo(x - 2, y + 2),
                Print(format!("  └{}┘ <", border))
            )?;
        } else {
            queue!(
                stdout,
                cursor::MoveTo(x - 2, y),
                Print(format!("  ┌{}┐", border)), // Added 2 spaces to align with selected state
                cursor::MoveTo(x - 2, y + 1),
                Print(format!("  │ {} │", self.label)),
                cursor::MoveTo(x - 2, y + 2),
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

    let mut name_input = InputInfoRow::new("Your name");
    let mut play_button = Button::new("PLAY", true);
    let mut exit_button = Button::new("EXIT", false);
    let mut selected_button = 0;

    loop {
        // Name input at top-left
        name_input.render(4, 2, stdout)?;

        // Center buttons horizontally
        let center_x = total_width / 2;
        play_button.render(center_x - 10, height / 2, stdout)?;
        exit_button.render(center_x + 5, height / 2, stdout)?;

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
