use crossterm::event::{
    KeyModifiers,
    KeyCode,
    Event,
    self,
};

use color_eyre::eyre::Result;

use std::process;

use ratatui::{
    style::{Color, Style, Stylize},
    text::{Line, Span},
    prelude::Backend,
    Terminal,
    Frame,
};

pub struct Prompt<'a> {
    prompt: &'a str,
    pointer: usize,
    input: String,
    done: bool,
}

impl<'a> Prompt<'a> {
    fn handle_event(&mut self) -> Result<()> {
        let Event::Key(k) = event::read()?
            else { return Ok(()); };

        let ctrl = k.modifiers.contains(
            KeyModifiers::CONTROL,
        );

        match k.code {
            KeyCode::Right => {
                if self.pointer < self.input.len() {
                    self.pointer += 1;
                }
            }
            KeyCode::Char('c') if ctrl => {
                process::exit(1);
            },
            KeyCode::Left => {
                if self.pointer > 0 {
                    self.pointer -= 1;
                }
            },
            KeyCode::Backspace => {
                if !self.input.is_empty() {
                    self.input.remove(
                        self.pointer - 1
                    );

                    if self.pointer > 0 {
                        self.pointer -= 1;
                    }
                }
            },
            KeyCode::Char(c) => {
                self.input.insert(
                    self.pointer,
                    c,
                );

                self.pointer += 1;
            },
            KeyCode::Enter => {
                self.done = true;
            }
            KeyCode::Tab => {
                self.input.insert(
                    self.pointer,
                    '\t',
                );

                self.pointer += 1;
            },
            _ => {}
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let (l, r) =
            self.input.split_at(self.pointer);

        let (m, r) = if !r.is_empty() {
            (
                &r[0..1],
                &r[1..],
            )
        } else {
            ("", "")
        };

        let line = Line::from(vec![
            Span::styled(
                format!("{} ", self.prompt),
                Style::default()
                    .bold(),
            ),
            Span::styled(
                "|> ",
                Style::default()
                    .bold()
                    .fg(Color::Blue)
            ),
            Span::styled(
                l,
                Style::default()
            ),
            Span::styled(
                m,
                Style::default()
                    .bg(Color::DarkGray),
            ),
            Span::styled(
                r,
                Style::default()
            ),
        ]);

        frame.render_widget(
            line,
            frame.area(),
        );
    }

    pub fn run(
        mut self,
        term: &mut Terminal<impl Backend>,
    ) -> Result<String> {
        loop {
            term.draw(|x| self.draw(x))?;
            self.handle_event()?;

            if self.done {
                return Ok(self.input);
            };
        }
    }

    pub const fn new(
        prompt: &'a str,
    ) -> Self {
        let input = String::new();

        Self {
            done: false,
            pointer: 0,
            prompt,
            input,
        }
    }

    pub fn prompt(
        term: &mut Terminal<impl Backend>,
        prompt: &'a str,
    ) -> Result<String> {
        Self::new(prompt)
            .run(term)
    }
}
