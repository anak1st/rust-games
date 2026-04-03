use crossterm::event::{KeyCode, KeyEvent};
use rand::RngExt;
use ratatui::{
    style::{Color, Style, Stylize},
    text::{Line, Text},
};

use crate::game::{
    Game, GameSize, GameStatus, Instruction, RenderBuffer, RenderGlyph, RenderMode, Renderable,
    Vec2,
};

const INSTRUCTIONS: [Instruction; 4] = [
    Instruction {
        label: " 移动 ",
        key: "<Left/Right/A/D>",
    },
    Instruction {
        label: " 旋转 ",
        key: "<Up/W>",
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
const BASE_DROP_INTERVAL: usize = 18;
const MIN_DROP_INTERVAL: usize = 4;
const SIMPLE_KICKS: [Vec2; 6] = [
    Vec2 { x: 0, y: 0 },
    Vec2 { x: -1, y: 0 },
    Vec2 { x: 1, y: 0 },
    Vec2 { x: 0, y: -1 },
    Vec2 { x: -2, y: 0 },
    Vec2 { x: 2, y: 0 },
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
    fn glyph(self) -> RenderGlyph {
        match self {
            Tetromino::I => RenderGlyph::new("I", "II"),
            Tetromino::O => RenderGlyph::new("O", "OO"),
            Tetromino::T => RenderGlyph::new("T", "TT"),
            Tetromino::S => RenderGlyph::new("S", "SS"),
            Tetromino::Z => RenderGlyph::new("Z", "ZZ"),
            Tetromino::J => RenderGlyph::new("J", "JJ"),
            Tetromino::L => RenderGlyph::new("L", "LL"),
        }
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
    fn cells(self, rotation: Rotation) -> [Vec2; 4] {
        match self {
            Tetromino::I => match rotation {
                Rotation::R0 => [
                    Vec2 { x: 0, y: 1 },
                    Vec2 { x: 1, y: 1 },
                    Vec2 { x: 2, y: 1 },
                    Vec2 { x: 3, y: 1 },
                ],
                Rotation::R90 => [
                    Vec2 { x: 2, y: 0 },
                    Vec2 { x: 2, y: 1 },
                    Vec2 { x: 2, y: 2 },
                    Vec2 { x: 2, y: 3 },
                ],
                Rotation::R180 => [
                    Vec2 { x: 0, y: 2 },
                    Vec2 { x: 1, y: 2 },
                    Vec2 { x: 2, y: 2 },
                    Vec2 { x: 3, y: 2 },
                ],
                Rotation::R270 => [
                    Vec2 { x: 1, y: 0 },
                    Vec2 { x: 1, y: 1 },
                    Vec2 { x: 1, y: 2 },
                    Vec2 { x: 1, y: 3 },
                ],
            },
            Tetromino::O => [
                Vec2 { x: 1, y: 0 },
                Vec2 { x: 2, y: 0 },
                Vec2 { x: 1, y: 1 },
                Vec2 { x: 2, y: 1 },
            ],
            Tetromino::T => match rotation {
                Rotation::R0 => [
                    Vec2 { x: 1, y: 0 },
                    Vec2 { x: 0, y: 1 },
                    Vec2 { x: 1, y: 1 },
                    Vec2 { x: 2, y: 1 },
                ],
                Rotation::R90 => [
                    Vec2 { x: 1, y: 0 },
                    Vec2 { x: 1, y: 1 },
                    Vec2 { x: 2, y: 1 },
                    Vec2 { x: 1, y: 2 },
                ],
                Rotation::R180 => [
                    Vec2 { x: 0, y: 1 },
                    Vec2 { x: 1, y: 1 },
                    Vec2 { x: 2, y: 1 },
                    Vec2 { x: 1, y: 2 },
                ],
                Rotation::R270 => [
                    Vec2 { x: 1, y: 0 },
                    Vec2 { x: 0, y: 1 },
                    Vec2 { x: 1, y: 1 },
                    Vec2 { x: 1, y: 2 },
                ],
            },
            Tetromino::S => match rotation {
                Rotation::R0 => [
                    Vec2 { x: 1, y: 0 },
                    Vec2 { x: 2, y: 0 },
                    Vec2 { x: 0, y: 1 },
                    Vec2 { x: 1, y: 1 },
                ],
                Rotation::R90 => [
                    Vec2 { x: 1, y: 0 },
                    Vec2 { x: 1, y: 1 },
                    Vec2 { x: 2, y: 1 },
                    Vec2 { x: 2, y: 2 },
                ],
                Rotation::R180 => [
                    Vec2 { x: 1, y: 1 },
                    Vec2 { x: 2, y: 1 },
                    Vec2 { x: 0, y: 2 },
                    Vec2 { x: 1, y: 2 },
                ],
                Rotation::R270 => [
                    Vec2 { x: 0, y: 0 },
                    Vec2 { x: 0, y: 1 },
                    Vec2 { x: 1, y: 1 },
                    Vec2 { x: 1, y: 2 },
                ],
            },
            Tetromino::Z => match rotation {
                Rotation::R0 => [
                    Vec2 { x: 0, y: 0 },
                    Vec2 { x: 1, y: 0 },
                    Vec2 { x: 1, y: 1 },
                    Vec2 { x: 2, y: 1 },
                ],
                Rotation::R90 => [
                    Vec2 { x: 2, y: 0 },
                    Vec2 { x: 1, y: 1 },
                    Vec2 { x: 2, y: 1 },
                    Vec2 { x: 1, y: 2 },
                ],
                Rotation::R180 => [
                    Vec2 { x: 0, y: 1 },
                    Vec2 { x: 1, y: 1 },
                    Vec2 { x: 1, y: 2 },
                    Vec2 { x: 2, y: 2 },
                ],
                Rotation::R270 => [
                    Vec2 { x: 1, y: 0 },
                    Vec2 { x: 0, y: 1 },
                    Vec2 { x: 1, y: 1 },
                    Vec2 { x: 0, y: 2 },
                ],
            },
            Tetromino::J => match rotation {
                Rotation::R0 => [
                    Vec2 { x: 0, y: 0 },
                    Vec2 { x: 0, y: 1 },
                    Vec2 { x: 1, y: 1 },
                    Vec2 { x: 2, y: 1 },
                ],
                Rotation::R90 => [
                    Vec2 { x: 1, y: 0 },
                    Vec2 { x: 2, y: 0 },
                    Vec2 { x: 1, y: 1 },
                    Vec2 { x: 1, y: 2 },
                ],
                Rotation::R180 => [
                    Vec2 { x: 0, y: 1 },
                    Vec2 { x: 1, y: 1 },
                    Vec2 { x: 2, y: 1 },
                    Vec2 { x: 2, y: 2 },
                ],
                Rotation::R270 => [
                    Vec2 { x: 1, y: 0 },
                    Vec2 { x: 1, y: 1 },
                    Vec2 { x: 0, y: 2 },
                    Vec2 { x: 1, y: 2 },
                ],
            },
            Tetromino::L => match rotation {
                Rotation::R0 => [
                    Vec2 { x: 2, y: 0 },
                    Vec2 { x: 0, y: 1 },
                    Vec2 { x: 1, y: 1 },
                    Vec2 { x: 2, y: 1 },
                ],
                Rotation::R90 => [
                    Vec2 { x: 1, y: 0 },
                    Vec2 { x: 1, y: 1 },
                    Vec2 { x: 1, y: 2 },
                    Vec2 { x: 2, y: 2 },
                ],
                Rotation::R180 => [
                    Vec2 { x: 0, y: 1 },
                    Vec2 { x: 1, y: 1 },
                    Vec2 { x: 2, y: 1 },
                    Vec2 { x: 0, y: 2 },
                ],
                Rotation::R270 => [
                    Vec2 { x: 0, y: 0 },
                    Vec2 { x: 1, y: 0 },
                    Vec2 { x: 1, y: 1 },
                    Vec2 { x: 1, y: 2 },
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
struct TetrisCell {
    point: Vec2,
    kind: Tetromino,
}

impl TetrisCell {
    /// 创建一个新的俄罗斯方块单元格。
    fn new(point: Vec2, kind: Tetromino) -> Self {
        Self { point, kind }
    }

    /// 返回平移后的单元格副本。
    fn transform(self, offset: Vec2) -> Self {
        Self {
            point: self.point.transform(offset),
            ..self
        }
    }
}

impl Renderable for TetrisCell {
    fn render(&self, buffer: &mut RenderBuffer, _frame: usize) {
        let style = match buffer.render_mode() {
            RenderMode::Single => Style::new().fg(self.kind.color()),
            RenderMode::Double => Style::new().fg(Color::Black).bg(self.kind.color()),
        };
        buffer.set(self.point, self.kind.glyph(), style);
    }
}

#[derive(Debug, Clone)]
struct TetrisPiece {
    kind: Tetromino,
    rotation: Rotation,
    origin: Vec2,
    cells: Vec<TetrisCell>,
}

impl TetrisPiece {
    /// 创建一个初始朝向的新方块。
    fn new(kind: Tetromino, origin: Vec2) -> Self {
        let mut piece = Self {
            kind,
            rotation: Rotation::R0,
            origin,
            cells: vec![],
        };
        piece.rebuild_cells();
        piece
    }

    /// 返回当前方块占用的棋盘坐标。
    fn points(&self) -> Vec<Vec2> {
        self.cells.iter().map(|cell| cell.point).collect()
    }

    /// 根据当前种类、朝向和原点重建单元格。
    fn rebuild_cells(&mut self) {
        self.cells = self
            .kind
            .cells(self.rotation)
            .into_iter()
            .map(|point| TetrisCell::new(point.transform(self.origin), self.kind))
            .collect();
    }

    /// 返回平移后的方块副本。
    fn transform(&self, offset: Vec2) -> Self {
        Self {
            kind: self.kind,
            rotation: self.rotation,
            origin: self.origin.transform(offset),
            cells: self
                .cells
                .iter()
                .map(|cell| cell.transform(offset))
                .collect(),
        }
    }

    /// 返回沿偏移量移动后的方块副本。
    fn moved(mut self, dx: isize, dy: isize) -> Self {
        self.origin = self.origin.transform(Vec2 { x: dx, y: dy });
        self.rebuild_cells();
        self
    }

    /// 返回顺时针旋转后的方块副本。
    fn rotated_right(mut self) -> Self {
        self.rotation = self.rotation.rotated_right();
        self.rebuild_cells();
        self
    }
}

impl Renderable for TetrisPiece {
    fn render(&self, buffer: &mut RenderBuffer, frame: usize) {
        for cell in &self.cells {
            cell.render(buffer, frame);
        }
    }
}

#[derive(Debug)]
pub struct GameTetris {
    size: GameSize,
    status: GameStatus,
    cells: Vec<TetrisCell>,
    active: Option<TetrisPiece>,
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
    pub fn new(size: GameSize, render_mode: RenderMode) -> Self {
        if size.width < BOARD_WIDTH || size.height < BOARD_HEIGHT {
            return Self {
                size,
                status: GameStatus::WindowTooSmall,
                cells: Self::empty_cells(),
                active: None,
                next: Tetromino::I,
                score: 0,
                lines: 0,
                level: 1,
                frame: 0,
                drop_interval: BASE_DROP_INTERVAL,
                buffer: RenderBuffer::new(size, render_mode),
            };
        }

        let mut game = Self {
            size,
            status: GameStatus::Running,
            cells: Self::empty_cells(),
            active: None,
            next: Tetromino::random(),
            score: 0,
            lines: 0,
            level: 1,
            frame: 0,
            drop_interval: BASE_DROP_INTERVAL,
            buffer: RenderBuffer::new(size, render_mode),
        };
        game.spawn_piece();
        game.update_symbols();
        game
    }

    /// 创建一个固定尺寸的空棋盘数据。
    fn empty_cells() -> Vec<TetrisCell> {
        vec![]
    }

    /// 返回内容区中用于绘制棋盘的左上角位置。
    fn board_origin(&self) -> Vec2 {
        Vec2 {
            x: self.size.width.saturating_sub(BOARD_WIDTH) as isize / 2,
            y: self.size.height.saturating_sub(BOARD_HEIGHT) as isize / 2,
        }
    }

    /// 判断给定坐标是否位于棋盘范围内。
    fn contains(&self, point: Vec2) -> bool {
        point.x >= 0
            && point.y >= 0
            && point.x < BOARD_WIDTH as isize
            && point.y < BOARD_HEIGHT as isize
    }

    /// 返回给定坐标上是否已经有已锁定方块。
    fn is_occupied(&self, point: Vec2) -> bool {
        if !self.contains(point) {
            return true;
        }
        self.cells.iter().any(|cell| cell.point == point)
    }

    /// 判断一个方块是否能合法放入当前棋盘。
    fn can_place(&self, piece: &TetrisPiece) -> bool {
        piece
            .points()
            .into_iter()
            .all(|point| self.contains(point) && !self.is_occupied(point))
    }

    /// 生成新的活动方块，并预先抽取下一个方块。
    fn spawn_piece(&mut self) {
        let piece = TetrisPiece::new(self.next, Vec2 { x: 3, y: 0 });
        self.next = Tetromino::random();
        if !self.can_place(&piece) {
            self.active = None;
            self.status = GameStatus::Lost;
            return;
        }
        self.active = Some(piece);
    }

    /// 尝试沿指定方向移动活动方块。
    fn try_move_active(&mut self, dx: isize, dy: isize) -> bool {
        let Some(active) = self.active.clone() else {
            return false;
        };
        let moved = active.moved(dx, dy);
        if !self.can_place(&moved) {
            return false;
        }
        self.active = Some(moved);
        true
    }

    /// 尝试顺时针旋转活动方块，并应用简单侧移修正。
    fn try_rotate_active(&mut self) -> bool {
        let Some(active) = self.active.clone() else {
            return false;
        };
        let rotated = active.rotated_right();
        for kick in SIMPLE_KICKS {
            let kicked = rotated.clone().moved(kick.x, kick.y);
            if !self.can_place(&kicked) {
                continue;
            }
            self.active = Some(kicked);
            return true;
        }
        false
    }

    /// 尝试沿活动方块向左移动一格。
    fn try_move_left(&mut self) -> bool {
        self.try_move_active(-1, 0)
    }

    /// 尝试沿活动方块向右移动一格。
    fn try_move_right(&mut self) -> bool {
        self.try_move_active(1, 0)
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
        let Some(active) = self.active.clone() else {
            return;
        };
        let mut dropped = active;
        loop {
            let next = dropped.clone().moved(0, 1);
            if !self.can_place(&next) {
                break;
            }
            dropped = next;
            self.score += 2;
        }
        self.active = Some(dropped);
        self.lock_active();
    }

    /// 将一个方块写入棋盘。
    fn lock_piece(&mut self, piece: TetrisPiece) {
        for cell in piece.cells {
            if !self.contains(cell.point) {
                continue;
            }
            self.cells.push(cell);
        }
    }

    /// 清除已满的行，并返回本次清除的行数。
    fn clear_lines(&mut self) -> usize {
        let mut row_counts = [0usize; BOARD_HEIGHT];
        for cell in &self.cells {
            row_counts[cell.point.y as usize] += 1;
        }

        let mut cleared_rows = [false; BOARD_HEIGHT];
        let mut cleared_count = 0;
        for (row, count) in row_counts.iter().enumerate() {
            if *count == BOARD_WIDTH {
                cleared_rows[row] = true;
                cleared_count += 1;
            }
        }

        if cleared_count == 0 {
            return 0;
        }

        let mut next_cells = Vec::with_capacity(self.cells.len());
        for cell in &self.cells {
            let row = cell.point.y as usize;
            if cleared_rows[row] {
                continue;
            }

            let mut drop_rows = 0;
            for cleared_row in row + 1..BOARD_HEIGHT {
                if cleared_rows[cleared_row] {
                    drop_rows += 1;
                }
            }

            next_cells.push(TetrisCell::new(
                cell.point.offset(0, drop_rows as isize),
                cell.kind,
            ));
        }

        self.cells = next_cells;
        cleared_count
    }

    /// 将当前活动方块写入棋盘并推进下一回合。
    fn lock_active(&mut self) {
        let Some(active) = self.active.take() else {
            return;
        };
        self.lock_piece(active);
        let cleared = self.clear_lines();
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
        self.buffer.set_bg_color(Color::DarkGray);

        let origin = self.board_origin();

        for y in 0..BOARD_HEIGHT {
            for x in 0..BOARD_WIDTH {
                self.buffer.set(
                    origin.offset(x as isize, y as isize),
                    RenderGlyph::new(".", ".."),
                    Style::new().fg(Color::DarkGray),
                );
            }
        }

        for cell in &self.cells {
            cell.transform(origin).render(&mut self.buffer, self.frame);
        }

        if let Some(active) = &self.active {
            active
                .transform(origin)
                .render(&mut self.buffer, self.frame);
        }
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
        let mut lines = Vec::with_capacity(4 + 7);
        lines.push(Line::from(vec![
            "下一个: ".into(),
            self.next.name().fg(self.next.color()),
        ]));

        let mut preview_buffer = RenderBuffer::new(
            GameSize {
                width: 4,
                height: 4,
            },
            self.buffer.render_mode(),
        );
        TetrisPiece::new(self.next, Vec2::default()).render(&mut preview_buffer, self.frame);
        lines.extend(preview_buffer.to_text().lines);

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
            KeyCode::Left | KeyCode::Char('a') => {
                self.try_move_left();
            }
            KeyCode::Right | KeyCode::Char('d') => {
                self.try_move_right();
            }
            KeyCode::Down | KeyCode::Char('s') => self.soft_drop(),
            KeyCode::Up | KeyCode::Char('w') => {
                self.try_rotate_active();
            }
            KeyCode::Enter => self.hard_drop(),
            _ => return,
        }
        self.update_symbols();
    }
}
