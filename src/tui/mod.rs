mod widgets;

use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::crossterm::{execute, terminal};

use std::io::{self, Stdout};
use std::time::Duration;

pub struct Command {
    cursor: usize,
    buffer: String,
}

impl Command {
    pub fn new() -> Command {
        Command {
            cursor: 0,
            buffer: String::new(),
        }
    }

    pub fn insert(&mut self, character: char) {
        self.buffer.insert(self.cursor, character);
    }

    pub fn remove(&mut self) {
        if !self.buffer.is_empty() {
            self.buffer.remove(self.cursor);
        }
    }

    pub fn clear(&mut self) {
        self.cursor = 0;

        self.buffer.drain(..);
    }
}

pub struct Windows {
    logging: Vec<String>,
    command: Command,
}

impl Windows {
    pub fn new() -> Windows {
        Windows {
            logging: Vec::new(),
            command: Command::new(),
        }
    }
}

pub struct Tui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    windows: Windows,
    should_close: bool,
}

impl Tui {
    pub fn new() -> Result<Tui, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Tui {
            terminal: Terminal::new(CrosstermBackend::new(io::stdout()))?,
            windows: Windows::new(),
            should_close: false,
        })
    }

    pub fn should_close(&mut self) -> bool { self.should_close }

    pub fn enter(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        terminal::enable_raw_mode()?;
        execute!(self.terminal.backend_mut(), terminal::EnterAlternateScreen)?;

        Ok(())
    }

    pub fn exit(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        terminal::disable_raw_mode()?;
        execute!(self.terminal.backend_mut(), terminal::LeaveAlternateScreen)?;

        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match key.kind {
            KeyEventKind::Press | KeyEventKind::Repeat => match key.code {
                KeyCode::Char(character) => {
                    self.windows.command.insert(character);
                },
                KeyCode::Backspace => {
                    self.windows.command.remove();
                },
                KeyCode::Enter => {
                    self.windows.command.clear();
                },
                KeyCode::Esc => {
                    self.should_close = true;
                },
                _ => {},
            },
            _ => {},
        }

        Ok(())
    }

    pub fn handle_input(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => {
                    self.handle_key(key)?;
                },
                _ => {},
            }
        }

        Ok(())
    }

    pub fn draw(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.terminal.draw(|frame| {
            widgets::draw(frame)
        })?;

        self.handle_input()?;

        Ok(())
    }
}


