pub mod counter;
pub mod snake;

use std::fmt::Debug;

use clap::ValueEnum;
use crossterm::event::KeyEvent;
use ratatui::{style::Color, text::Text};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum GameKind {
    Counter,
    Snake,
}

impl GameKind {
    pub fn name(self) -> &'static str {
        match self {
            GameKind::Counter => "计数器",
            GameKind::Snake => "贪吃蛇",
        }
    }
}

pub const GAMES: [GameKind; 2] = [GameKind::Counter, GameKind::Snake];

#[derive(Debug, Clone, Copy)]
pub struct Instruction {
    pub label: &'static str,
    pub key: &'static str,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GameSize {
    pub width: u16,
    pub height: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GameStatus {
    #[default]
    Idle,
    Main,
    Running,
    Paused,
    Won,
    Lost,
    WindowTooSmall,
}

impl GameStatus {
    pub fn label(self) -> &'static str {
        match self {
            GameStatus::Idle => "空闲",
            GameStatus::Main => "主界面",
            GameStatus::Running => "进行中",
            GameStatus::Paused => "已暂停",
            GameStatus::Won => "已胜利",
            GameStatus::Lost => "已失败",
            GameStatus::WindowTooSmall => "窗口太小",
        }
    }

    pub fn color(self) -> Color {
        match self {
            GameStatus::Idle => Color::Gray,
            GameStatus::Main => Color::Cyan,
            GameStatus::Running => Color::Green,
            GameStatus::Paused => Color::Yellow,
            GameStatus::Won => Color::Green,
            GameStatus::Lost => Color::Red,
            GameStatus::WindowTooSmall => Color::LightMagenta,
        }
    }
}

/// 可嵌入应用外壳中的游戏所需遵循的最小约定。
pub trait Game: Debug {
    /// 更新游戏逻辑。
    fn update(&mut self);
    /// 返回当前游戏状态。
    fn status(&self) -> GameStatus;
    /// 返回当前游戏内容，用于渲染内容区域。
    fn render_content(&self) -> Text<'static>;
    /// 返回当前游戏状态面板，用于渲染状态区域。
    fn render_status(&self) -> Text<'static>;
    /// 返回底部区域使用的帮助说明。
    fn instructions(&self) -> Vec<Instruction>;
    /// 处理游戏按键事件并更新内部状态。
    fn handle_key_event(&mut self, key_event: KeyEvent);
}
