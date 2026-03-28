use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    style::Stylize,
    text::{Line, Text},
};

use crate::game::{Game, GameSize, GameStatus, Instruction};

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
pub struct GameCounter {
    counter: i64,
    size: GameSize,
    status: GameStatus,
}

impl GameCounter {
    /// 创建一个新的计数器游戏实例。
    pub fn new(size: GameSize) -> Self {
        Self {
            counter: 0,
            size,
            status: GameStatus::Running,
        }
    }
}

impl Game for GameCounter {
    /// 更新计数器游戏逻辑。
    fn update(&mut self) {}

    /// 返回计数器游戏当前状态。
    fn status(&self) -> GameStatus {
        self.status
    }

    /// 渲染计数器游戏的内容区域。
    fn render_content(&self) -> Text<'static> {
        Text::from(vec![Line::from(vec![
            "Value: ".into(),
            self.counter.to_string().yellow(),
        ])])
    }

    /// 渲染计数器游戏的状态区域。
    fn render_status(&self) -> Text<'static> {
        Text::from(vec![
            Line::from(vec!["Counter: ".into(), self.counter.to_string().yellow()]),
            Line::from(vec![
                "Size: ".into(),
                format!("{} x {}", self.size.width, self.size.height).yellow(),
            ]),
        ])
    }

    /// 返回计数器游戏的帮助说明。
    fn instructions(&self) -> Vec<Instruction> {
        INSTRUCTIONS.to_vec()
    }

    /// 处理计数器游戏的按键输入。
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Left => self.counter -= 1,
            KeyCode::Right => self.counter += 1,
            _ => {}
        }
    }
}
