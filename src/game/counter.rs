use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    style::Stylize,
    text::{Line, Text},
};

use crate::game::{Game, Instruction};

const INSTRUCTIONS: [Instruction; 4] = [
    Instruction {
        label: " Decrement ",
        key: "<Left>",
    },
    Instruction {
        label: " Increment ",
        key: "<Right>",
    },
    Instruction {
        label: " Pause ",
        key: "<P>",
    },
    Instruction {
        label: " Restart ",
        key: "<R>",
    },
];

#[derive(Debug, Default)]
pub struct CounterGame {
    counter: i64,
}

impl Game for CounterGame {
    fn title(&self) -> &'static str {
        "Counter Demo"
    }

    fn content(&self) -> Text<'static> {
        Text::from(vec![
            Line::from("Counter Demo".bold()),
            Line::from(""),
            Line::from("这里先放最小的游戏区域。"),
            Line::from(vec!["Value: ".into(), self.counter.to_string().yellow()]),
        ])
    }

    fn status(&self) -> Text<'static> {
        Text::from(vec![
            Line::from("Game Status".bold()),
            Line::from(""),
            Line::from(vec!["Counter: ".into(), self.counter.to_string().yellow()]),
        ])
    }

    fn instructions(&self) -> &'static [Instruction] {
        &INSTRUCTIONS
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('r') => {
                self.counter = 0;
            }
            KeyCode::Left => self.counter -= 1,
            KeyCode::Right => self.counter += 1,
            _ => {}
        }
    }
}
