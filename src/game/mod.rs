pub mod counter;
pub mod snake;

use std::fmt::Debug;

use clap::ValueEnum;
use crossterm::event::KeyEvent;
use ratatui::style::{Color, Style};
use ratatui::text::Text;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Point {
    pub x: isize,
    pub y: isize,
}

impl Point {
    /// 返回当前点到目标点的曼哈顿距离。
    pub fn distance_to(self, other: Point) -> isize {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }

    /// 返回沿给定偏移量移动后的坐标。
    pub fn offset(self, dx: isize, dy: isize) -> Point {
        Point {
            x: self.x + dx,
            y: self.y + dy,
        }
    }

    /// 返回沿给定方向移动一步后的坐标。
    pub fn step(self, direction: Direction) -> Point {
        match direction {
            Direction::Up => self.offset(0, -1),
            Direction::Down => self.offset(0, 1),
            Direction::Left => self.offset(-1, 0),
            Direction::Right => self.offset(1, 0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    /// 返回方向在界面中展示的符号。
    pub fn label(self) -> &'static str {
        match self {
            Direction::Up => "↑",
            Direction::Down => "↓",
            Direction::Left => "←",
            Direction::Right => "→",
        }
    }

    /// 返回当前方向的反方向。
    pub fn opposite(self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }

    /// 判断两个方向是否彼此相反。
    pub fn is_opposite(self, other: Direction) -> bool {
        self.opposite() == other
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Instruction {
    pub label: &'static str,
    pub key: &'static str,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GameSize {
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GameStatus {
    #[default]
    Idle,
    Main,
    Ready,
    Running,
    Paused,
    Won,
    Lost,
    WindowTooSmall,
}

impl GameStatus {
    /// 返回游戏状态在界面中展示的文案。
    pub fn label(self) -> &'static str {
        match self {
            GameStatus::Idle => "空闲",
            GameStatus::Main => "主界面",
            GameStatus::Ready => "准备",
            GameStatus::Running => "进行中",
            GameStatus::Paused => "已暂停",
            GameStatus::Won => "已胜利",
            GameStatus::Lost => "已失败",
            GameStatus::WindowTooSmall => "窗口太小",
        }
    }

    /// 返回游戏状态在界面中使用的样式。
    pub fn style(self) -> Style {
        let color = match self {
            GameStatus::Idle => Color::Gray,
            GameStatus::Main => Color::Cyan,
            GameStatus::Ready => Color::LightGreen,
            GameStatus::Running => Color::Green,
            GameStatus::Paused => Color::Yellow,
            GameStatus::Won => Color::Green,
            GameStatus::Lost => Color::Red,
            GameStatus::WindowTooSmall => Color::LightMagenta,
        };
        Style::new().fg(color)
    }
}

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
