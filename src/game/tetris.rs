use crossterm::event::{KeyCode, KeyEvent};
use rand::RngExt;
use ratatui::{
    style::{Color, Stylize},
    text::{Line, Span, Text},
};

use crate::game::{Game, GameSize, GameStatus, Instruction, Point, RenderBuffer};

const INSTRUCTIONS: [Instruction; 4] = [
    Instruction {
        label: " 移动 ",
        key: "<Left/Right/A/D>",
    },
    Instruction {
        label: " 旋转 ",
        key: "<Up/W/X>",
    },
    Instruction {
        label: " 软降 ",
        key: "<Down/S>",
    },
    Instruction {
        label: " 硬降 ",
        key: "<Enter>",
    },
];

const BOARD_WIDTH: usize = 10;
const BOARD_HEIGHT: usize = 20;
const BOARD_RENDER_WIDTH: usize = BOARD_WIDTH + 2;
const BOARD_RENDER_HEIGHT: usize = BOARD_HEIGHT + 2;
const MIN_WIDTH: usize = BOARD_RENDER_WIDTH;
const MIN_HEIGHT: usize = BOARD_RENDER_HEIGHT;
const PREVIEW_SIZE: usize = 4;
const BASE_DROP_INTERVAL: usize = 18;
const MIN_DROP_INTERVAL: usize = 4;
const SIMPLE_KICKS: [Point; 6] = [
    Point { x: 0, y: 0 },
    Point { x: -1, y: 0 },
    Point { x: 1, y: 0 },
    Point { x: 0, y: -1 },
    Point { x: -2, y: 0 },
    Point { x: 2, y: 0 },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Rotation {
    R0,
    R90,
    R180,
    R270,
}

impl Rotation {
    /// 返回顺时针旋转一次后的朝向。
    fn rotated_right(self) -> Self {
        match self {
            Rotation::R0 => Rotation::R90,
            Rotation::R90 => Rotation::R180,
            Rotation::R180 => Rotation::R270,
            Rotation::R270 => Rotation::R0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tetromino {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

impl Tetromino {
    /// 返回当前方块在界面中展示的名称。
    fn name(self) -> &'static str {
        match self {
            Tetromino::I => "I",
            Tetromino::O => "O",
            Tetromino::T => "T",
            Tetromino::S => "S",
            Tetromino::Z => "Z",
            Tetromino::J => "J",
            Tetromino::L => "L",
        }
    }

    /// 返回当前方块在棋盘上显示的符号。
    fn symbol(self) -> &'static str {
        self.name()
    }

    /// 返回当前方块在棋盘上使用的颜色。
    fn color(self) -> Color {
        match self {
            Tetromino::I => Color::Cyan,
            Tetromino::O => Color::Yellow,
            Tetromino::T => Color::Magenta,
            Tetromino::S => Color::Green,
            Tetromino::Z => Color::Red,
            Tetromino::J => Color::Blue,
            Tetromino::L => Color::LightYellow,
        }
    }

    /// 返回当前方块在指定朝向下的局部坐标。
    fn cells(self, rotation: Rotation) -> [Point; 4] {
        match self {
            Tetromino::I => match rotation {
                Rotation::R0 => [
                    Point { x: 0, y: 1 },
                    Point { x: 1, y: 1 },
                    Point { x: 2, y: 1 },
                    Point { x: 3, y: 1 },
                ],
                Rotation::R90 => [
                    Point { x: 2, y: 0 },
                    Point { x: 2, y: 1 },
                    Point { x: 2, y: 2 },
                    Point { x: 2, y: 3 },
                ],
                Rotation::R180 => [
                    Point { x: 0, y: 2 },
                    Point { x: 1, y: 2 },
                    Point { x: 2, y: 2 },
                    Point { x: 3, y: 2 },
                ],
                Rotation::R270 => [
                    Point { x: 1, y: 0 },
                    Point { x: 1, y: 1 },
                    Point { x: 1, y: 2 },
                    Point { x: 1, y: 3 },
                ],
            },
            Tetromino::O => [
                Point { x: 1, y: 0 },
                Point { x: 2, y: 0 },
                Point { x: 1, y: 1 },
                Point { x: 2, y: 1 },
            ],
            Tetromino::T => match rotation {
                Rotation::R0 => [
                    Point { x: 1, y: 0 },
                    Point { x: 0, y: 1 },
                    Point { x: 1, y: 1 },
                    Point { x: 2, y: 1 },
                ],
                Rotation::R90 => [
                    Point { x: 1, y: 0 },
                    Point { x: 1, y: 1 },
                    Point { x: 2, y: 1 },
                    Point { x: 1, y: 2 },
                ],
                Rotation::R180 => [
                    Point { x: 0, y: 1 },
                    Point { x: 1, y: 1 },
                    Point { x: 2, y: 1 },
                    Point { x: 1, y: 2 },
                ],
                Rotation::R270 => [
                    Point { x: 1, y: 0 },
                    Point { x: 0, y: 1 },
                    Point { x: 1, y: 1 },
                    Point { x: 1, y: 2 },
                ],
            },
            Tetromino::S => match rotation {
                Rotation::R0 => [
                    Point { x: 1, y: 0 },
                    Point { x: 2, y: 0 },
                    Point { x: 0, y: 1 },
                    Point { x: 1, y: 1 },
                ],
                Rotation::R90 => [
                    Point { x: 1, y: 0 },
                    Point { x: 1, y: 1 },
                    Point { x: 2, y: 1 },
                    Point { x: 2, y: 2 },
                ],
                Rotation::R180 => [
                    Point { x: 1, y: 1 },
                    Point { x: 2, y: 1 },
                    Point { x: 0, y: 2 },
                    Point { x: 1, y: 2 },
                ],
                Rotation::R270 => [
                    Point { x: 0, y: 0 },
                    Point { x: 0, y: 1 },
                    Point { x: 1, y: 1 },
                    Point { x: 1, y: 2 },
                ],
            },
            Tetromino::Z => match rotation {
                Rotation::R0 => [
                    Point { x: 0, y: 0 },
                    Point { x: 1, y: 0 },
                    Point { x: 1, y: 1 },
                    Point { x: 2, y: 1 },
                ],
                Rotation::R90 => [
                    Point { x: 2, y: 0 },
                    Point { x: 1, y: 1 },
                    Point { x: 2, y: 1 },
                    Point { x: 1, y: 2 },
                ],
                Rotation::R180 => [
                    Point { x: 0, y: 1 },
                    Point { x: 1, y: 1 },
                    Point { x: 1, y: 2 },
                    Point { x: 2, y: 2 },
                ],
                Rotation::R270 => [
                    Point { x: 1, y: 0 },
                    Point { x: 0, y: 1 },
                    Point { x: 1, y: 1 },
                    Point { x: 0, y: 2 },
                ],
            },
            Tetromino::J => match rotation {
                Rotation::R0 => [
                    Point { x: 0, y: 0 },
                    Point { x: 0, y: 1 },
                    Point { x: 1, y: 1 },
                    Point { x: 2, y: 1 },
                ],
                Rotation::R90 => [
                    Point { x: 1, y: 0 },
                    Point { x: 2, y: 0 },
                    Point { x: 1, y: 1 },
                    Point { x: 1, y: 2 },
                ],
                Rotation::R180 => [
                    Point { x: 0, y: 1 },
                    Point { x: 1, y: 1 },
                    Point { x: 2, y: 1 },
                    Point { x: 2, y: 2 },
                ],
                Rotation::R270 => [
                    Point { x: 1, y: 0 },
                    Point { x: 1, y: 1 },
                    Point { x: 0, y: 2 },
                    Point { x: 1, y: 2 },
                ],
            },
            Tetromino::L => match rotation {
                Rotation::R0 => [
                    Point { x: 2, y: 0 },
                    Point { x: 0, y: 1 },
                    Point { x: 1, y: 1 },
                    Point { x: 2, y: 1 },
                ],
                Rotation::R90 => [
                    Point { x: 1, y: 0 },
                    Point { x: 1, y: 1 },
                    Point { x: 1, y: 2 },
                    Point { x: 2, y: 2 },
                ],
                Rotation::R180 => [
                    Point { x: 0, y: 1 },
                    Point { x: 1, y: 1 },
                    Point { x: 2, y: 1 },
                    Point { x: 0, y: 2 },
                ],
                Rotation::R270 => [
                    Point { x: 0, y: 0 },
                    Point { x: 1, y: 0 },
                    Point { x: 1, y: 1 },
                    Point { x: 1, y: 2 },
                ],
            },
        }
    }

    /// 随机生成一个俄罗斯方块类型。
    fn random() -> Self {
        let kinds = [
            Tetromino::I,
            Tetromino::O,
            Tetromino::T,
            Tetromino::S,
            Tetromino::Z,
            Tetromino::J,
            Tetromino::L,
        ];
        let mut rng = rand::rng();
        let index = rng.random_range(0..kinds.len());
        kinds[index]
    }
}

#[derive(Debug, Clone, Copy)]
struct Piece {
    kind: Tetromino,
    rotation: Rotation,
    origin: Point,
}

impl Piece {
    /// 创建一个初始朝向的新方块。
    fn new(kind: Tetromino) -> Self {
        Self {
            kind,
            rotation: Rotation::R0,
            origin: Point { x: 3, y: 0 },
        }
    }

    /// 返回当前方块占用的棋盘坐标。
    fn points(self) -> [Point; 4] {
        self.kind
            .cells(self.rotation)
            .map(|point| point.offset(self.origin.x, self.origin.y))
    }

    /// 返回沿偏移量移动后的方块副本。
    fn moved(self, dx: isize, dy: isize) -> Self {
        Self {
            origin: self.origin.offset(dx, dy),
            ..self
        }
    }

    /// 返回顺时针旋转后的方块副本。
    fn rotated_right(self) -> Self {
        Self {
            rotation: self.rotation.rotated_right(),
            ..self
        }
    }
}

#[derive(Debug)]
struct Board {
    cells: Vec<Vec<Option<Tetromino>>>,
}

impl Board {
    /// 创建一个固定尺寸的空棋盘。
    fn new() -> Self {
        Self {
            cells: vec![vec![None; BOARD_WIDTH]; BOARD_HEIGHT],
        }
    }

    /// 判断给定坐标是否位于棋盘范围内。
    fn contains(&self, point: Point) -> bool {
        point.x >= 0
            && point.y >= 0
            && point.x < BOARD_WIDTH as isize
            && point.y < BOARD_HEIGHT as isize
    }

    /// 返回给定坐标上是否已经有已锁定方块。
    fn is_occupied(&self, point: Point) -> bool {
        if !self.contains(point) {
            return true;
        }
        self.cells[point.y as usize][point.x as usize].is_some()
    }

    /// 将一个方块写入棋盘。
    fn lock_piece(&mut self, piece: Piece) {
        for point in piece.points() {
            if !self.contains(point) {
                continue;
            }
            self.cells[point.y as usize][point.x as usize] = Some(piece.kind);
        }
    }

    /// 清除已满的行，并返回本次清除的行数。
    fn clear_lines(&mut self) -> usize {
        let mut remaining_rows = Vec::with_capacity(BOARD_HEIGHT);
        let mut cleared = 0;
        for row in &self.cells {
            if row.iter().all(Option::is_some) {
                cleared += 1;
                continue;
            }
            remaining_rows.push(row.clone());
        }
        while remaining_rows.len() < BOARD_HEIGHT {
            remaining_rows.insert(0, vec![None; BOARD_WIDTH]);
        }
        self.cells = remaining_rows;
        cleared
    }
}

#[derive(Debug)]
pub struct GameTetris {
    size: GameSize,
    status: GameStatus,
    board: Board,
    active: Option<Piece>,
    next: Tetromino,
    score: usize,
    lines: usize,
    level: usize,
    frame: usize,
    drop_interval: usize,
    buffer: RenderBuffer,
}

impl GameTetris {
    /// 创建一个新的俄罗斯方块游戏实例。
    pub fn new(size: GameSize) -> Self {
        if size.width < MIN_WIDTH || size.height < MIN_HEIGHT {
            return Self {
                size,
                status: GameStatus::WindowTooSmall,
                board: Board::new(),
                active: None,
                next: Tetromino::I,
                score: 0,
                lines: 0,
                level: 1,
                frame: 0,
                drop_interval: BASE_DROP_INTERVAL,
                buffer: RenderBuffer::new(size),
            };
        }

        let mut game = Self {
            size,
            status: GameStatus::Running,
            board: Board::new(),
            active: None,
            next: Tetromino::random(),
            score: 0,
            lines: 0,
            level: 1,
            frame: 0,
            drop_interval: BASE_DROP_INTERVAL,
            buffer: RenderBuffer::new(size),
        };
        game.spawn_piece();
        game.update_symbols();
        game
    }

    /// 返回内容区中用于绘制棋盘的左上角位置。
    fn board_origin(&self) -> Point {
        Point {
            x: self.size.width.saturating_sub(BOARD_RENDER_WIDTH) as isize / 2,
            y: self.size.height.saturating_sub(BOARD_RENDER_HEIGHT) as isize / 2,
        }
    }

    /// 判断一个方块是否能合法放入当前棋盘。
    fn can_place(&self, piece: Piece) -> bool {
        piece
            .points()
            .into_iter()
            .all(|point| self.board.contains(point) && !self.board.is_occupied(point))
    }

    /// 生成新的活动方块，并预先抽取下一个方块。
    fn spawn_piece(&mut self) {
        let piece = Piece::new(self.next);
        self.next = Tetromino::random();
        if !self.can_place(piece) {
            self.active = None;
            self.status = GameStatus::Lost;
            return;
        }
        self.active = Some(piece);
    }

    /// 尝试沿指定方向移动活动方块。
    fn try_move_active(&mut self, dx: isize, dy: isize) -> bool {
        let Some(active) = self.active else {
            return false;
        };
        let moved = active.moved(dx, dy);
        if !self.can_place(moved) {
            return false;
        }
        self.active = Some(moved);
        true
    }

    /// 尝试顺时针旋转活动方块，并应用简单侧移修正。
    fn try_rotate_active(&mut self) -> bool {
        let Some(active) = self.active else {
            return false;
        };
        let rotated = active.rotated_right();
        for kick in SIMPLE_KICKS {
            let kicked = rotated.moved(kick.x, kick.y);
            if !self.can_place(kicked) {
                continue;
            }
            self.active = Some(kicked);
            return true;
        }
        false
    }

    /// 让活动方块向下移动一格，无法下落时立即锁定。
    fn soft_drop(&mut self) {
        if self.try_move_active(0, 1) {
            self.score += 1;
            return;
        }
        self.lock_active();
    }

    /// 将活动方块一路下落到底部并立即锁定。
    fn hard_drop(&mut self) {
        let Some(active) = self.active else {
            return;
        };
        let mut dropped = active;
        while self.can_place(dropped.moved(0, 1)) {
            dropped = dropped.moved(0, 1);
            self.score += 2;
        }
        self.active = Some(dropped);
        self.lock_active();
    }

    /// 将当前活动方块写入棋盘并推进下一回合。
    fn lock_active(&mut self) {
        let Some(active) = self.active.take() else {
            return;
        };
        self.board.lock_piece(active);
        let cleared = self.board.clear_lines();
        self.apply_line_clear(cleared);
        if self.status == GameStatus::Running {
            self.spawn_piece();
        }
    }

    /// 根据消行数量更新分数、等级和下落速度。
    fn apply_line_clear(&mut self, cleared: usize) {
        if cleared == 0 {
            return;
        }
        self.lines += cleared;
        let points = match cleared {
            1 => 100,
            2 => 300,
            3 => 500,
            4 => 800,
            _ => 0,
        };
        self.score += points * self.level;
        self.level = self.lines / 10 + 1;
        self.drop_interval = BASE_DROP_INTERVAL
            .saturating_sub((self.level - 1) * 2)
            .max(MIN_DROP_INTERVAL);
    }

    /// 推进一次自动下落逻辑。
    fn step_gravity(&mut self) {
        if self.try_move_active(0, 1) {
            return;
        }
        self.lock_active();
    }

    /// 重建当前帧的渲染缓存。
    fn update_symbols(&mut self) {
        self.buffer.clear();
        self.fill_background();
        self.draw_board_frame();
        self.draw_board_cells();
        self.draw_active_piece();
    }

    /// 用空格填满整个内容区，避免未使用区域出现默认占位符。
    fn fill_background(&mut self) {
        for y in 0..self.size.height as isize {
            for x in 0..self.size.width as isize {
                self.buffer.set(Point { x, y }, " ", Color::Reset);
            }
        }
    }

    /// 绘制棋盘边框。
    fn draw_board_frame(&mut self) {
        let origin = self.board_origin();
        for x in 0..BOARD_RENDER_WIDTH as isize {
            let symbol = if x == 0 || x == BOARD_RENDER_WIDTH as isize - 1 {
                "+"
            } else {
                "-"
            };
            self.buffer
                .set(origin.offset(x, 0), symbol, Color::DarkGray);
            self.buffer.set(
                origin.offset(x, BOARD_RENDER_HEIGHT as isize - 1),
                symbol,
                Color::DarkGray,
            );
        }
        for y in 1..BOARD_RENDER_HEIGHT as isize - 1 {
            self.buffer.set(origin.offset(0, y), "|", Color::DarkGray);
            self.buffer.set(
                origin.offset(BOARD_RENDER_WIDTH as isize - 1, y),
                "|",
                Color::DarkGray,
            );
        }
    }

    /// 绘制所有已经锁定的格子和空棋盘底色。
    fn draw_board_cells(&mut self) {
        let origin = self.board_origin().offset(1, 1);
        for y in 0..BOARD_HEIGHT {
            for x in 0..BOARD_WIDTH {
                let point = origin.offset(x as isize, y as isize);
                let Some(kind) = self.board.cells[y][x] else {
                    self.buffer.set(point, ".", Color::DarkGray);
                    continue;
                };
                self.buffer.set(point, kind.symbol(), kind.color());
            }
        }
    }

    /// 将当前活动方块叠加到渲染缓存上。
    fn draw_active_piece(&mut self) {
        let Some(active) = self.active else {
            return;
        };
        let origin = self.board_origin().offset(1, 1);
        for point in active.points() {
            self.buffer.set(
                origin.offset(point.x, point.y),
                active.kind.symbol(),
                active.kind.color(),
            );
        }
    }

    /// 返回一个方块预览区域的文本行。
    fn piece_preview_lines(
        label: &'static str,
        kind: Tetromino,
        rotation: Rotation,
    ) -> Vec<Line<'static>> {
        let mut preview = [[false; PREVIEW_SIZE]; PREVIEW_SIZE];
        for point in kind.cells(rotation) {
            if point.x < 0 || point.y < 0 {
                continue;
            }
            let x = point.x as usize;
            let y = point.y as usize;
            if x >= PREVIEW_SIZE || y >= PREVIEW_SIZE {
                continue;
            }
            preview[y][x] = true;
        }

        let mut lines = Vec::with_capacity(PREVIEW_SIZE + 1);
        lines.push(Line::from(vec![label.into(), kind.name().fg(kind.color())]));
        for row in preview {
            let spans: Vec<_> = row
                .into_iter()
                .map(|filled| {
                    if filled {
                        Span::styled(kind.symbol(), kind.color())
                    } else {
                        Span::styled(".", Color::DarkGray)
                    }
                })
                .collect();
            lines.push(Line::from(spans));
        }
        lines
    }
}

impl Game for GameTetris {
    /// 推进一帧俄罗斯方块逻辑。
    fn update(&mut self) {
        if self.status != GameStatus::Running {
            return;
        }
        self.frame += 1;
        if self.frame % self.drop_interval == 0 {
            self.step_gravity();
        }
        self.update_symbols();
    }

    /// 返回俄罗斯方块当前状态。
    fn status(&self) -> GameStatus {
        self.status
    }

    /// 渲染俄罗斯方块的内容区域。
    fn render_content(&self) -> Text<'static> {
        if self.status == GameStatus::WindowTooSmall {
            return Text::from("俄罗斯方块区域太小");
        }
        self.buffer.to_text()
    }

    /// 渲染俄罗斯方块的状态区域。
    fn render_status(&self) -> Text<'static> {
        let mut lines = Self::piece_preview_lines("下一个: ", self.next, Rotation::R0);
        lines.extend([
            Line::from(vec!["分数: ".into(), self.score.to_string().yellow()]),
            Line::from(vec!["行数: ".into(), self.lines.to_string().green()]),
            Line::from(vec!["等级: ".into(), self.level.to_string().cyan()]),
            Line::from(vec![
                "棋盘: ".into(),
                format!("{BOARD_WIDTH} x {BOARD_HEIGHT}").into(),
            ]),
            Line::from(vec![
                "速度: ".into(),
                format!("{} / 30", self.drop_interval).into(),
            ]),
            Line::from(vec![
                "尺寸: ".into(),
                format!("{} x {}", self.size.width, self.size.height).into(),
            ]),
        ]);
        Text::from(lines)
    }

    /// 返回俄罗斯方块的帮助说明。
    fn instructions(&self) -> Vec<Instruction> {
        INSTRUCTIONS.to_vec()
    }

    /// 处理俄罗斯方块的操作输入。
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('A') => {
                self.try_move_active(-1, 0);
            }
            KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('D') => {
                self.try_move_active(1, 0);
            }
            KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => self.soft_drop(),
            KeyCode::Up
            | KeyCode::Char('w')
            | KeyCode::Char('W')
            | KeyCode::Char('x')
            | KeyCode::Char('X') => {
                self.try_rotate_active();
            }
            KeyCode::Enter => self.hard_drop(),
            _ => return,
        }
        self.update_symbols();
    }
}
