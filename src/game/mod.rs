pub mod counter;
pub mod snake;
pub mod tetris;

use std::fmt::Debug;

use clap::ValueEnum;
use crossterm::event::KeyEvent;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span, Text};

pub const EMPTY_SYMBOL: &str = ".";
pub const EMPTY_COLOR: Color = Color::DarkGray;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
pub enum RenderMode {
    Single,
    #[default]
    Double,
}

impl RenderMode {
    /// 返回当前渲染模式下每个格子占用的终端列数。
    pub const fn cell_width(self) -> usize {
        match self {
            RenderMode::Single => 1,
            RenderMode::Double => 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderGlyph {
    pub single: &'static str,
    pub double: &'static str,
}

impl RenderGlyph {
    /// 创建一组单字符和双字符显示符号。
    pub const fn new(single: &'static str, double: &'static str) -> Self {
        Self { single, double }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RenderCell {
    pub glyph: RenderGlyph,
    pub style: Style,
}

impl RenderCell {
    pub const fn empty() -> Self {
        Self {
            glyph: RenderGlyph::new(EMPTY_SYMBOL, ".."),
            style: Style::new().fg(EMPTY_COLOR),
        }
    }
}

#[derive(Debug)]
pub struct RenderBuffer {
    cells: Vec<Vec<RenderCell>>,
    mode: RenderMode,
}

impl RenderBuffer {
    pub fn new(size: GameSize, mode: RenderMode) -> Self {
        Self {
            cells: vec![vec![RenderCell::empty(); size.width]; size.height],
            mode,
        }
    }

    pub const fn mode(&self) -> RenderMode {
        self.mode
    }

    pub fn clear(&mut self) {
        for row in &mut self.cells {
            row.fill(RenderCell::empty());
        }
    }

    pub fn set(&mut self, point: Point, glyph: RenderGlyph, style: Style) {
        if point.x < 0 || point.y < 0 {
            return;
        }
        let Some(row) = self.cells.get_mut(point.y as usize) else {
            return;
        };
        let Some(cell) = row.get_mut(point.x as usize) else {
            return;
        };
        *cell = RenderCell { glyph, style };
    }

    pub fn symbol_at(&self, point: Point) -> &'static str {
        if point.x < 0 || point.y < 0 {
            return EMPTY_SYMBOL;
        }
        self.cells
            .get(point.y as usize)
            .and_then(|row| row.get(point.x as usize))
            .map(|cell| cell.glyph.single)
            .unwrap_or(EMPTY_SYMBOL)
    }

    pub fn to_text(&self) -> Text<'static> {
        let mut lines = Vec::with_capacity(self.cells.len());
        for row in &self.cells {
            let spans: Vec<_> = row
                .iter()
                .map(|cell| {
                    let symbol = match self.mode {
                        RenderMode::Single => cell.glyph.single,
                        RenderMode::Double => cell.glyph.double,
                    };
                    Span::styled(symbol, cell.style)
                })
                .collect();
            lines.push(Line::from(spans));
        }
        Text::from(lines)
    }
}

pub trait Renderable {
    fn render(&self, buffer: &mut RenderBuffer, frame: usize);
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
            GameStatus::Ready => Color::Cyan,
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
    Tetris,
}

impl GameKind {
    /// 返回游戏类型在界面中展示的名称。
    pub fn name(self) -> &'static str {
        match self {
            GameKind::Counter => "计数器",
            GameKind::Snake => "贪吃蛇",
            GameKind::Tetris => "俄罗斯方块",
        }
    }
}

pub const GAMES: [GameKind; 3] = [GameKind::Counter, GameKind::Snake, GameKind::Tetris];

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
