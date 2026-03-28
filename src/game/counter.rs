use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    style::Stylize,
    text::{Line, Text},
};

use crate::game::{Game, GameSize, Instruction};

const INSTRUCTIONS: [Instruction; 2] = [
    Instruction {
        label: " 减少 ",
        key: "<Left>",
    },
    Instruction {
        label: " 增加 ",
        key: "<Right>",
    },
];

#[derive(Debug, Default)]
pub struct CounterGame {
    counter: i64,
    size: GameSize,
}

impl CounterGame {
    pub fn new(size: GameSize) -> Self {
        Self { counter: 0, size }
    }
}

impl Game for CounterGame {
    fn title(&self) -> String {
        "Counter Demo".to_string()
    }

    fn content(&self) -> Text<'static> {
        Text::from(vec![Line::from(vec![
            "Value: ".into(),
            self.counter.to_string().yellow(),
        ])])
    }

    fn status(&self) -> Text<'static> {
        Text::from(vec![
            Line::from(vec!["Counter: ".into(), self.counter.to_string().yellow()]),
            Line::from(vec![
                "Size: ".into(),
                format!("{} x {}", self.size.width, self.size.height).yellow(),
            ]),
        ])
    }

    fn instructions(&self) -> Vec<Instruction> {
        INSTRUCTIONS.to_vec()
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Left => self.counter -= 1,
            KeyCode::Right => self.counter += 1,
            _ => {}
        }
    }
}
