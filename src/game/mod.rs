pub mod counter;

use std::fmt::Debug;

use clap::ValueEnum;
use crossterm::event::KeyEvent;
use ratatui::text::Text;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum GameKind {
    Counter,
}

pub const GAMES: [GameKind; 1] = [GameKind::Counter];

#[derive(Debug, Clone, Copy)]
pub struct Instruction {
    pub label: &'static str,
    pub key: &'static str,
}

/// 可嵌入应用外壳中的游戏所需遵循的最小约定。
pub trait Game: Debug {
    /// 返回显示在应用标题栏中的标题。
    fn title(&self) -> &'static str;
    /// 返回当前游戏内容，用于渲染内容区域。
    fn content(&self) -> Text<'static>;
    /// 返回当前游戏状态，用于渲染状态区域。
    fn status(&self) -> Text<'static>;
    /// 返回底部区域使用的帮助说明。
    fn instructions(&self) -> &'static [Instruction];
    /// 处理游戏按键事件并更新内部状态。
    fn handle_key_event(&mut self, key_event: KeyEvent);
}
