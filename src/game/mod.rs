pub mod common;
pub mod counter;
pub mod snake;

use std::fmt::Debug;

use clap::ValueEnum;
use crossterm::event::KeyEvent;
use ratatui::text::Text;

pub use self::common::{Direction, GameSize, GameStatus, Instruction, Point};

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum GameKind {
    Counter,
    Snake,
}

impl GameKind {
    /// 返回游戏类型在界面中展示的名称。
    pub fn name(self) -> &'static str {
        match self {
            GameKind::Counter => "计数器",
            GameKind::Snake => "贪吃蛇",
        }
    }
}

pub const GAMES: [GameKind; 2] = [GameKind::Counter, GameKind::Snake];

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
