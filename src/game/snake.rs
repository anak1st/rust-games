use crossterm::event::{KeyCode, KeyEvent};
use rand::RngExt;
use ratatui::{
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
};

use crate::game::{Game, GameSize, GameStatus, Instruction};

const INSTRUCTIONS: [Instruction; 1] = [Instruction {
    label: " 转向 ",
    key: "<Arrows/WASD>",
}];

const MIN_WIDTH: u16 = 12;
const MIN_HEIGHT: u16 = 8;
const FRAMES_PER_STEP: u8 = 4;
const FOOD_COUNT: usize = 3;
const AI_COUNT: usize = 4;
const DEAD_WAIT_STEPS: u8 = 10;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Point {
    x: i16,
    y: i16,
}

impl Point {
    /// 返回当前点到目标点的曼哈顿距离。
    fn distance_to(self, other: Point) -> i16 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }

    /// 返回沿给定方向移动一步后的坐标。
    fn step(self, direction: Direction) -> Point {
        match direction {
            Direction::Up => Point {
                x: self.x,
                y: self.y - 1,
            },
            Direction::Down => Point {
                x: self.x,
                y: self.y + 1,
            },
            Direction::Left => Point {
                x: self.x - 1,
                y: self.y,
            },
            Direction::Right => Point {
                x: self.x + 1,
                y: self.y,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    /// 返回方向在界面中展示的符号。
    fn label(self) -> &'static str {
        match self {
            Direction::Up => "↑",
            Direction::Down => "↓",
            Direction::Left => "←",
            Direction::Right => "→",
        }
    }

    /// 判断两个方向是否彼此相反。
    fn is_opposite(self, other: Direction) -> bool {
        matches!(
            (self, other),
            (Direction::Up, Direction::Down)
                | (Direction::Down, Direction::Up)
                | (Direction::Left, Direction::Right)
                | (Direction::Right, Direction::Left)
        )
    }

    /// 返回当前方向的反方向。
    fn opposite(self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SnakeState {
    Alive,
    Dead { remaining: u8 },
}

#[derive(Debug)]
struct SpawnConfig {
    body: Vec<Point>,
    direction: Direction,
}

#[derive(Debug)]
struct Snake {
    body: Vec<Point>,
    direction: Direction,
    head_symbol: &'static str,
    body_symbol: &'static str,
    head_color: Color,
    body_color: Color,
    score: usize,
    pending_growth: usize,
    state: SnakeState,
}

impl Snake {
    /// 创建玩家控制的蛇实例。
    fn new_player(body: Vec<Point>) -> Self {
        Self {
            body,
            direction: Direction::Right,
            head_symbol: "@",
            body_symbol: "o",
            head_color: Color::White,
            body_color: Color::White,
            score: 0,
            pending_growth: 0,
            state: SnakeState::Alive,
        }
    }

    /// 创建 AI 控制的蛇实例。
    fn new_ai(index: usize, body: Vec<Point>) -> Self {
        let head_symbol = match index {
            0 => "A",
            1 => "B",
            2 => "C",
            3 => "D",
            _ => "Z",
        };
        let body_symbol = match index {
            0 => "a",
            1 => "b",
            2 => "c",
            3 => "d",
            _ => "z",
        };
        let (head_color, body_color) = match index {
            0 => (Color::LightGreen, Color::Green),
            1 => (Color::LightYellow, Color::Yellow),
            2 => (Color::LightRed, Color::Red),
            3 => (Color::LightBlue, Color::Blue),
            _ => (Color::Gray, Color::DarkGray),
        };
        Self {
            body,
            direction: Direction::Left,
            head_symbol,
            body_symbol,
            head_color,
            body_color,
            score: 0,
            pending_growth: 0,
            state: SnakeState::Alive,
        }
    }

    /// 返回蛇头所在的位置。
    fn head(&self) -> Point {
        self.body[0]
    }

    /// 返回蛇当前的身体长度。
    fn len(&self) -> usize {
        self.body.len()
    }

    /// 返回蛇当前累计分数。
    fn score(&self) -> usize {
        self.score
    }

    /// 返回蛇当前是否仍然存活。
    fn is_alive(&self) -> bool {
        self.state == SnakeState::Alive
    }

    /// 根据当前方向计算下一帧蛇头的位置。
    fn next_head(&self) -> Point {
        self.head().step(self.direction)
    }

    /// 判断给定位置是否被整条蛇占用。
    fn contains(&self, point: Point) -> bool {
        self.is_alive() && self.body.contains(&point)
    }

    /// 判断给定位置是否被蛇头占用。
    fn head_contains(&self, point: Point) -> bool {
        self.is_alive() && self.head() == point
    }

    /// 判断给定位置是否被蛇身占用。
    fn body_contains(&self, point: Point) -> bool {
        self.is_alive() && self.body.len() > 1 && self.body[1..].contains(&point)
    }

    /// 返回蛇本次移动后需要保留的身体长度。
    fn body_len_after_move(&self, ate_food: bool) -> usize {
        if self.pending_growth > 0 || ate_food {
            self.len()
        } else {
            self.len() - 1
        }
    }

    /// 让蛇沿当前方向前进一步。
    fn forward(&mut self) {
        let next_head = self.next_head();
        self.body.insert(0, next_head);
        if self.pending_growth > 0 {
            self.pending_growth -= 1;
        } else {
            self.body.pop();
        }
    }

    /// 更新蛇当前的移动方向。
    fn set_direction(&mut self, direction: Direction) {
        self.direction = direction;
    }

    /// 让蛇吃到一个食物。
    fn eat_food(&mut self) {
        self.score += 1;
        self.pending_growth += 1;
    }

    /// 将蛇标记为死亡，并清空当前身体。
    fn mark_dead(&mut self) {
        self.body.clear();
        self.pending_growth = 0;
        self.state = SnakeState::Dead {
            remaining: DEAD_WAIT_STEPS,
        };
    }

    /// 推进一次死亡等待，并返回是否可以尝试重生。
    fn tick_dead(&mut self) -> bool {
        match &mut self.state {
            SnakeState::Alive => false,
            SnakeState::Dead { remaining } => {
                if *remaining > 0 {
                    *remaining -= 1;
                }
                *remaining == 0
            }
        }
    }

    /// 使用新的出生点配置重生。
    fn respawn(&mut self, spawn: SpawnConfig) {
        self.body = spawn.body;
        self.direction = spawn.direction;
        self.score = 0;
        self.pending_growth = 0;
        self.state = SnakeState::Alive;
    }
}

#[derive(Debug)]
pub struct GameSnake {
    size: GameSize,
    status: GameStatus,
    player: Snake,
    snakes: Vec<Snake>,
    foods: Vec<Point>,
    frame: u8,
}

impl GameSnake {
    /// 创建一个新的贪吃蛇游戏实例。
    pub fn new(size: GameSize) -> Self {
        if size.width < MIN_WIDTH || size.height < MIN_HEIGHT {
            return Self {
                size,
                status: GameStatus::WindowTooSmall,
                player: Snake::new_player(vec![]),
                snakes: (0..AI_COUNT)
                    .map(|index| Snake::new_ai(index, vec![]))
                    .collect(),
                foods: vec![],
                frame: 0,
            };
        }
        let center_x = (size.width / 2) as i16;
        let center_y = (size.height / 2) as i16;
        let player = Snake::new_player(vec![
            Point {
                x: center_x + 1,
                y: center_y,
            },
            Point {
                x: center_x,
                y: center_y,
            },
            Point {
                x: center_x - 1,
                y: center_y,
            },
        ]);
        let mut game = Self {
            size,
            status: GameStatus::Running,
            player,
            snakes: (0..AI_COUNT)
                .map(|index| {
                    let mut snake = Snake::new_ai(index, vec![]);
                    snake.respawn(Self::corner_spawn_config(size, index));
                    snake
                })
                .collect(),
            foods: vec![],
            frame: 0,
        };
        game.fill_foods();
        game
    }

    /// 返回四个角落使用的固定出生点配置。
    fn corner_spawn_config(size: GameSize, index: usize) -> SpawnConfig {
        let right = size.width as i16 - 1;
        let bottom = size.height as i16 - 1;
        match index {
            0 => SpawnConfig {
                body: vec![
                    Point { x: 0, y: 0 },
                    Point { x: 0, y: 1 },
                    Point { x: 0, y: 2 },
                ],
                direction: Direction::Right,
            },
            1 => SpawnConfig {
                body: vec![
                    Point { x: right, y: 0 },
                    Point { x: right, y: 1 },
                    Point { x: right, y: 2 },
                ],
                direction: Direction::Left,
            },
            2 => SpawnConfig {
                body: vec![
                    Point { x: 0, y: bottom },
                    Point {
                        x: 0,
                        y: bottom - 1,
                    },
                    Point {
                        x: 0,
                        y: bottom - 2,
                    },
                ],
                direction: Direction::Right,
            },
            _ => SpawnConfig {
                body: vec![
                    Point {
                        x: right,
                        y: bottom,
                    },
                    Point {
                        x: right,
                        y: bottom - 1,
                    },
                    Point {
                        x: right,
                        y: bottom - 2,
                    },
                ],
                direction: Direction::Left,
            },
        }
    }

    /// 判断给定位置是否被任意一条蛇占用。
    fn is_occupied(&self, point: Point) -> bool {
        self.player.contains(point) || self.snakes.iter().any(|snake| snake.contains(point))
    }

    /// 判断一组出生点是否可以安全放下一条蛇。
    fn can_spawn_body(&self, body: &[Point]) -> bool {
        body.iter().all(|point| {
            self.is_inside(*point) && !self.is_occupied(*point) && !self.foods.contains(point)
        })
    }

    /// 收集当前可以使用的所有出生点配置。
    fn spawn_candidates(&self) -> Vec<SpawnConfig> {
        let mut candidates = Vec::new();

        for index in 0..AI_COUNT {
            let spawn = Self::corner_spawn_config(self.size, index);
            if self.can_spawn_body(&spawn.body) {
                candidates.push(spawn);
            }
        }

        for y in 0..self.size.height as i16 {
            for x in 0..self.size.width as i16 {
                let head = Point { x, y };
                for direction in [
                    Direction::Up,
                    Direction::Down,
                    Direction::Left,
                    Direction::Right,
                ] {
                    let mut body = Vec::with_capacity(3);
                    body.push(head);
                    let mut current = head;
                    for _ in 1..3 {
                        current = current.step(direction.opposite());
                        body.push(current);
                    }
                    if !self.can_spawn_body(&body) {
                        continue;
                    }
                    candidates.push(SpawnConfig { body, direction });
                }
            }
        }

        candidates
    }

    /// 为一条死亡的 AI 蛇随机选择重生位置。
    fn respawn_snake(&mut self, snake_index: usize) {
        let candidates = self.spawn_candidates();
        if candidates.is_empty() {
            return;
        }
        let mut rng = rand::rng();
        let spawn_index = rng.random_range(0..candidates.len());
        self.snakes[snake_index].respawn(candidates.into_iter().nth(spawn_index).unwrap());
    }

    /// 为棋盘补齐缺少的食物数量。
    fn fill_foods(&mut self) {
        let mut empty_points = Vec::new();
        for y in 0..self.size.height as i16 {
            for x in 0..self.size.width as i16 {
                let point = Point { x, y };
                if self.is_occupied(point) || self.foods.contains(&point) {
                    continue;
                }
                empty_points.push(point);
            }
        }
        let mut rng = rand::rng();
        let missing_foods = FOOD_COUNT.saturating_sub(self.foods.len());
        let food_count = missing_foods.min(empty_points.len());
        for _ in 0..food_count {
            let index = rng.random_range(0..empty_points.len());
            self.foods.push(empty_points.swap_remove(index));
        }
    }

    /// 判断位置是否在棋盘内。
    fn is_inside(&self, point: Point) -> bool {
        point.x >= 0
            && point.y >= 0
            && point.x < self.size.width as i16
            && point.y < self.size.height as i16
    }

    /// 判断一条蛇移动到目标位置后是否会失败。
    ///
    /// 判断顺序如下：
    /// 1. 先判断是否越界。越界时直接失败，不再继续后续判断。
    /// 2. 再判断是否撞到自己。这里不会一刀切地检查整条身体，
    ///    而是先根据这一步是否会增长，算出移动后仍然保留的身体长度。
    ///    如果这一步不会增长，尾巴会同步前移，所以允许蛇头落到“当前尾巴所在格”。
    /// 3. 最后判断是否撞到其他蛇。这里直接传入“自己”和“其他蛇”，
    ///    这样调用方可以明确地决定当前这次移动到底要检查哪些对象。
    fn hits_obstacle(
        &self,
        snake: &Snake,
        other_snakes: &[&Snake],
        next_head: Point,
        ate_food: bool,
    ) -> bool {
        if !self.is_inside(next_head) {
            return true;
        }
        let body_len = snake.body_len_after_move(ate_food);
        if snake.body[..body_len].contains(&next_head) {
            return true;
        }
        for other_snake in other_snakes {
            if other_snake.contains(next_head) {
                return true;
            }
        }
        false
    }

    /// 为指定 AI 蛇选择下一步移动方向。
    fn update_ai_direction(&mut self, snake_index: usize) {
        if !self.snakes[snake_index].is_alive() {
            return;
        }
        let Some(target) = self
            .foods
            .iter()
            .min_by_key(|food| self.snakes[snake_index].head().distance_to(**food))
            .copied()
        else {
            return;
        };
        let mut directions = vec![
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ];
        directions.sort_by_key(|direction| {
            let next_head = next_head(self.snakes[snake_index].head(), *direction);
            next_head.distance_to(target)
        });
        for direction in directions {
            if direction.is_opposite(self.snakes[snake_index].direction) {
                continue;
            }
            let next_head = next_head(self.snakes[snake_index].head(), direction);
            let ate_food = self.foods.contains(&next_head);
            let blocked = {
                let snake = &self.snakes[snake_index];
                let mut other_snakes = Vec::with_capacity(self.snakes.len());
                other_snakes.push(&self.player);
                for (index, other_snake) in self.snakes.iter().enumerate() {
                    if index != snake_index {
                        other_snakes.push(other_snake);
                    }
                }
                self.hits_obstacle(snake, &other_snakes, next_head, ate_food)
            };
            if blocked {
                continue;
            }
            self.snakes[snake_index].set_direction(direction);
            return;
        }
    }

    /// 推进玩家蛇的一次移动。
    fn update_player(&mut self) {
        let next_head = self.player.next_head();
        let food_index = self.foods.iter().position(|food| *food == next_head);
        let ate_food = food_index.is_some();
        let other_snakes = self.snakes.iter().collect::<Vec<_>>();
        if self.hits_obstacle(&self.player, &other_snakes, next_head, ate_food) {
            self.status = GameStatus::Lost;
            return;
        }
        if let Some(food_index) = food_index {
            self.player.eat_food();
            self.foods.swap_remove(food_index);
        }
        self.player.forward();
        if food_index.is_some() {
            self.fill_foods();
        }
    }

    /// 推进所有 AI 蛇的一次移动。
    fn update_snakes(&mut self) {
        for snake_index in 0..self.snakes.len() {
            if !self.snakes[snake_index].is_alive() {
                if self.snakes[snake_index].tick_dead() {
                    self.respawn_snake(snake_index);
                }
                continue;
            }
            self.update_ai_direction(snake_index);
            let next_head = self.snakes[snake_index].next_head();
            let food_index = self.foods.iter().position(|food| *food == next_head);
            let ate_food = food_index.is_some();
            let blocked = {
                let snake = &self.snakes[snake_index];
                let mut other_snakes = Vec::with_capacity(self.snakes.len());
                other_snakes.push(&self.player);
                for (index, other_snake) in self.snakes.iter().enumerate() {
                    if index != snake_index {
                        other_snakes.push(other_snake);
                    }
                }
                self.hits_obstacle(snake, &other_snakes, next_head, ate_food)
            };
            if blocked {
                self.snakes[snake_index].mark_dead();
                continue;
            }
            if let Some(food_index) = food_index {
                self.snakes[snake_index].eat_food();
                self.foods.swap_remove(food_index);
            }
            self.snakes[snake_index].forward();
            if food_index.is_some() {
                self.fill_foods();
            }
        }
    }

    /// 渲染一个棋盘格子。
    fn render_cell(&self, point: Point) -> Span<'static> {
        let mut symbol = ".";
        let mut color = Color::DarkGray;
        if self.foods.contains(&point) {
            symbol = "*";
            color = Color::Magenta;
        }
        for snake in &self.snakes {
            if snake.body_contains(point) {
                symbol = snake.body_symbol;
                color = snake.body_color;
                break;
            }
        }
        for snake in &self.snakes {
            if snake.head_contains(point) {
                symbol = snake.head_symbol;
                color = snake.head_color;
                break;
            }
        }
        if self.player.body_contains(point) {
            symbol = self.player.body_symbol;
            color = self.player.body_color;
        }
        if self.player.head_contains(point) {
            symbol = self.player.head_symbol;
            color = self.player.head_color;
        }
        Span::styled(symbol, Style::new().fg(color))
    }
}

/// 根据给定方向计算下一步坐标。
fn next_head(head: Point, direction: Direction) -> Point {
    match direction {
        Direction::Up => Point {
            x: head.x,
            y: head.y - 1,
        },
        Direction::Down => Point {
            x: head.x,
            y: head.y + 1,
        },
        Direction::Left => Point {
            x: head.x - 1,
            y: head.y,
        },
        Direction::Right => Point {
            x: head.x + 1,
            y: head.y,
        },
    }
}

impl Game for GameSnake {
    /// 推进一帧贪吃蛇游戏逻辑。
    fn update(&mut self) {
        if self.status != GameStatus::Running {
            return;
        }
        self.frame += 1;
        if self.frame < FRAMES_PER_STEP {
            return;
        }
        self.frame = 0;
        self.update_snakes();
        self.update_player();
    }

    /// 返回贪吃蛇游戏当前状态。
    fn status(&self) -> GameStatus {
        self.status
    }

    /// 渲染贪吃蛇游戏的内容区域。
    fn render_content(&self) -> Text<'static> {
        if self.status == GameStatus::WindowTooSmall {
            return Text::from("贪吃蛇区域太小");
        }
        let mut lines = Vec::with_capacity(self.size.height as usize);
        for y in 0..self.size.height as i16 {
            let mut spans = Vec::with_capacity(self.size.width as usize);
            for x in 0..self.size.width as i16 {
                spans.push(self.render_cell(Point { x, y }));
            }
            lines.push(Line::from(spans));
        }
        Text::from(lines)
    }

    /// 渲染贪吃蛇游戏的状态区域。
    fn render_status(&self) -> Text<'static> {
        Text::from(vec![
            Line::from(vec![
                "玩家方向: ".into(),
                self.player.direction.label().fg(self.player.head_color),
            ]),
            Line::from(vec![
                "玩家分数: ".into(),
                self.player.score().to_string().fg(self.player.head_color),
            ]),
            Line::from(self.snakes.iter().enumerate().fold(
                vec!["敌人方向: ".into()],
                |mut spans, (index, snake)| {
                    if index > 0 {
                        spans.push(" ".into());
                    }
                    spans.push(snake.direction.label().fg(snake.head_color));
                    spans
                },
            )),
            Line::from(self.snakes.iter().enumerate().fold(
                vec!["敌人分数: ".into()],
                |mut spans, (index, snake)| {
                    if index > 0 {
                        spans.push(" ".into());
                    }
                    spans.push(snake.score().to_string().fg(snake.head_color));
                    spans
                },
            )),
            Line::from(vec!["食物: ".into(), self.foods.len().to_string().green()]),
            Line::from(vec![
                "尺寸: ".into(),
                format!("{} x {}", self.size.width, self.size.height).into(),
            ]),
            Line::from(vec![
                "速度: ".into(),
                format!("{} / {}", FRAMES_PER_STEP, 30).into(),
            ]),
        ])
    }

    /// 返回贪吃蛇游戏的帮助说明。
    fn instructions(&self) -> Vec<Instruction> {
        INSTRUCTIONS.to_vec()
    }

    /// 处理贪吃蛇游戏的方向输入。
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        let next_direction = match key_event.code {
            KeyCode::Up | KeyCode::Char('w') => Some(Direction::Up),
            KeyCode::Down | KeyCode::Char('s') => Some(Direction::Down),
            KeyCode::Left | KeyCode::Char('a') => Some(Direction::Left),
            KeyCode::Right | KeyCode::Char('d') => Some(Direction::Right),
            _ => None,
        };
        let Some(next_direction) = next_direction else {
            return;
        };
        if next_direction.is_opposite(self.player.direction) {
            return;
        }
        self.player.set_direction(next_direction);
    }
}
