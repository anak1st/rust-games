use crossterm::event::{KeyCode, KeyEvent};
use rand::RngExt;
use ratatui::{
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
};

use crate::game::{Direction, Game, GameSize, GameStatus, Instruction, Point};

const INSTRUCTIONS: [Instruction; 2] = [
    Instruction {
        label: " 转向 ",
        key: "<Arrows/WASD>",
    },
    Instruction {
        label: " 切换控制 ",
        key: "<I>",
    },
];

const MIN_WIDTH: u16 = 12;
const MIN_HEIGHT: u16 = 8;
const FRAMES_PER_STEP: u8 = 4;
const FOOD_COUNT: usize = 3;
const AI_COUNT: usize = 4;
const DEAD_WAIT_STEPS: u8 = 10;
const AI_ROAM_CHANCE_PERCENT: u8 = 5;
const AI_ROAM_STEPS: u8 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SnakeController {
    Manual,
    Ai(AiState),
}

impl SnakeController {
    /// 返回控制模式在界面中展示的文案。
    fn label(self) -> &'static str {
        match self {
            SnakeController::Manual => "手动",
            SnakeController::Ai(_) => "AI",
        }
    }

    /// 返回该控制模式是否接受玩家输入。
    fn accepts_manual_input(self) -> bool {
        matches!(self, SnakeController::Manual)
    }

    /// 在手动和 AI 控制之间切换。
    fn toggled(self) -> SnakeController {
        match self {
            SnakeController::Manual => SnakeController::Ai(AiState::default()),
            SnakeController::Ai(_) => SnakeController::Manual,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct AiState {
    roaming_steps: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SnakeState {
    Alive,
    Dead { remaining: u8 },
}

#[derive(Debug)]
struct SnakeSpawn {
    body: Vec<Point>,
    direction: Direction,
}

#[derive(Debug, Clone, Copy)]
struct Food {
    point: Point,
    growth: usize,
    symbol: &'static str,
    color: Color,
}

impl Food {
    /// 创建一个默认食物。
    fn new(point: Point) -> Self {
        Self {
            point,
            growth: 1,
            symbol: "*",
            color: Color::Magenta,
        }
    }
}

#[derive(Debug)]
struct Snake {
    body: Vec<Point>,
    direction: Direction,
    controller: SnakeController,
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
    fn new_player(body: Vec<Point>, controller: SnakeController) -> Self {
        Self {
            body,
            direction: Direction::Right,
            controller,
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
            controller: SnakeController::Ai(AiState::default()),
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

    /// 返回蛇当前的控制模式。
    fn controller(&self) -> SnakeController {
        self.controller
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

    /// 更新蛇当前的移动方向。
    fn set_direction(&mut self, direction: Direction) {
        self.direction = direction;
    }

    /// 让蛇沿当前方向前进一步。
    fn advance(&mut self) {
        self.advance_roaming();
        let next_head = self.next_head();
        self.body.insert(0, next_head);
        if self.pending_growth > 0 {
            self.pending_growth -= 1;
        } else {
            self.body.pop();
        }
    }

    /// 返回 AI 蛇当前是否仍处于漫游状态。
    fn is_roaming(&self) -> bool {
        match self.controller {
            SnakeController::Ai(state) => state.roaming_steps > 0,
            SnakeController::Manual => false,
        }
    }

    /// 让 AI 蛇进入一段固定步数的漫游状态。
    fn start_roaming(&mut self, steps: u8) {
        if let SnakeController::Ai(state) = &mut self.controller {
            if state.roaming_steps == 0 {
                state.roaming_steps = steps;
            }
        }
    }

    /// 消耗一次漫游步数。
    fn advance_roaming(&mut self) {
        if let SnakeController::Ai(state) = &mut self.controller {
            if state.roaming_steps > 0 {
                state.roaming_steps -= 1;
            }
        }
    }

    /// 让蛇吃到一个食物，并按配置增加分数和长度。
    fn eat(&mut self, food: Food) {
        self.score += food.growth;
        self.pending_growth += food.growth;
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
    fn spawn(&mut self, spawn: SnakeSpawn) {
        self.body = spawn.body;
        self.direction = spawn.direction;
        self.score = 0;
        self.pending_growth = 0;
        self.state = SnakeState::Alive;
        if let SnakeController::Ai(state) = &mut self.controller {
            *state = AiState::default();
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum SnakeSlot {
    Player,
    Enemy(usize),
}

#[derive(Debug)]
pub struct GameSnake {
    size: GameSize,
    status: GameStatus,
    player: Snake,
    snakes: Vec<Snake>,
    foods: Vec<Food>,
    frame: u8,
}

impl GameSnake {
    /// 创建一个新的贪吃蛇游戏实例。
    pub fn new(size: GameSize) -> Self {
        if size.width < MIN_WIDTH || size.height < MIN_HEIGHT {
            return Self {
                size,
                status: GameStatus::WindowTooSmall,
                player: Snake::new_player(vec![], SnakeController::Manual),
                snakes: vec![],
                foods: vec![],
                frame: 0,
            };
        }
        let mut game = Self {
            size,
            status: GameStatus::Running,
            player: Self::spawn_player(size),
            snakes: vec![],
            foods: vec![],
            frame: 0,
        };
        for index in 0..AI_COUNT {
            game.snakes.push(Snake::new_ai(index, vec![]));
            if !game.spawn_snake(index) {
                game.snakes[index].mark_dead();
            }
        }
        while game.foods.len() < FOOD_COUNT {
            game.spawn_food();
        }
        game
    }

    /// helper

    /// 判断给定位置是否被任意一条蛇占用。
    fn is_occupied(&self, point: Point) -> bool {
        self.player.contains(point) || self.snakes.iter().any(|snake| snake.contains(point))
    }

    /// 判断位置是否在棋盘内。
    fn is_inside(&self, point: Point) -> bool {
        point.x >= 0
            && point.y >= 0
            && point.x < self.size.width as i16
            && point.y < self.size.height as i16
    }

    /// 判断一组位置是否可以安全放下一条蛇。
    fn is_valid_points(&self, points: &[Point]) -> bool {
        points.iter().all(|point| {
            self.is_inside(*point)
                && !self.is_occupied(*point)
                && !self.foods.iter().any(|food| food.point == *point)
        })
    }

    /// 返回指定槽位对应的蛇实例。
    fn snake(&self, slot: SnakeSlot) -> &Snake {
        match slot {
            SnakeSlot::Player => &self.player,
            SnakeSlot::Enemy(index) => &self.snakes[index],
        }
    }

    /// 返回指定槽位对应的可变蛇实例。
    fn snake_mut(&mut self, slot: SnakeSlot) -> &mut Snake {
        match slot {
            SnakeSlot::Player => &mut self.player,
            SnakeSlot::Enemy(index) => &mut self.snakes[index],
        }
    }

    // spawn

    /// 创建玩家初始蛇。
    fn spawn_player(size: GameSize) -> Snake {
        let center_x = (size.width / 2) as i16;
        let center_y = (size.height / 2) as i16;
        Snake::new_player(
            vec![
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
            ],
            SnakeController::Manual,
        )
    }

    /// 为一条 AI 蛇随机选择出生位置。
    fn spawn_snake(&mut self, snake_index: usize) -> bool {
        let right = self.size.width as i16 - 1;
        let bottom = self.size.height as i16 - 1;
        let mut corner_candidates = Vec::new();
        for spawn in [
            SnakeSpawn {
                body: vec![
                    Point { x: 0, y: 0 },
                    Point { x: 0, y: 1 },
                    Point { x: 0, y: 2 },
                ],
                direction: Direction::Right,
            },
            SnakeSpawn {
                body: vec![
                    Point { x: 0, y: 0 },
                    Point { x: 1, y: 0 },
                    Point { x: 2, y: 0 },
                ],
                direction: Direction::Down,
            },
            SnakeSpawn {
                body: vec![
                    Point { x: right, y: 0 },
                    Point { x: right, y: 1 },
                    Point { x: right, y: 2 },
                ],
                direction: Direction::Left,
            },
            SnakeSpawn {
                body: vec![
                    Point { x: right, y: 0 },
                    Point { x: right - 1, y: 0 },
                    Point { x: right - 2, y: 0 },
                ],
                direction: Direction::Down,
            },
            SnakeSpawn {
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
            SnakeSpawn {
                body: vec![
                    Point { x: 0, y: bottom },
                    Point { x: 1, y: bottom },
                    Point { x: 2, y: bottom },
                ],
                direction: Direction::Up,
            },
            SnakeSpawn {
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
            SnakeSpawn {
                body: vec![
                    Point {
                        x: right,
                        y: bottom,
                    },
                    Point {
                        x: right - 1,
                        y: bottom,
                    },
                    Point {
                        x: right - 2,
                        y: bottom,
                    },
                ],
                direction: Direction::Up,
            },
        ] {
            if self.is_valid_points(&spawn.body) {
                corner_candidates.push(spawn);
            }
        }
        if !corner_candidates.is_empty() {
            let mut rng = rand::rng();
            let spawn_index = rng.random_range(0..corner_candidates.len());
            self.snakes[snake_index].spawn(corner_candidates.swap_remove(spawn_index));
            return true;
        }

        let mut candidates = Vec::new();
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
                    if !self.is_valid_points(&body) {
                        continue;
                    }
                    candidates.push(SnakeSpawn { body, direction });
                }
            }
        }
        if candidates.is_empty() {
            return false;
        }
        let mut rng = rand::rng();
        let spawn_index = rng.random_range(0..candidates.len());
        self.snakes[snake_index].spawn(candidates.swap_remove(spawn_index));
        true
    }

    /// 在任意空位上随机生成一个食物。
    fn spawn_food(&mut self) {
        if self.foods.len() >= FOOD_COUNT {
            return;
        }
        let mut empty_points = Vec::new();
        for y in 0..self.size.height as i16 {
            for x in 0..self.size.width as i16 {
                let point = Point { x, y };
                if self.is_occupied(point) || self.foods.iter().any(|food| food.point == point) {
                    continue;
                }
                empty_points.push(point);
            }
        }
        if empty_points.is_empty() {
            return;
        }
        let mut rng = rand::rng();
        let index = rng.random_range(0..empty_points.len());
        self.foods.push(Food::new(empty_points.swap_remove(index)));
    }

    // update

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

    /// 收集给定蛇下一步可安全移动的方向。
    fn get_snake_safe_directions(&self, slot: SnakeSlot) -> Vec<Direction> {
        let snake = self.snake(slot);
        let mut directions = vec![
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ];
        directions.retain(|direction| {
            if direction.is_opposite(snake.direction) {
                return false;
            }
            let next_head = snake.head().step(*direction);
            let ate_food = self.foods.iter().any(|food| food.point == next_head);
            let blocked = match slot {
                SnakeSlot::Player => {
                    let other_snakes = self.snakes.iter().collect::<Vec<_>>();
                    self.hits_obstacle(snake, &other_snakes, next_head, ate_food)
                }
                SnakeSlot::Enemy(snake_index) => {
                    let mut other_snakes = Vec::with_capacity(self.snakes.len());
                    other_snakes.push(&self.player);
                    for (index, other_snake) in self.snakes.iter().enumerate() {
                        if index != snake_index {
                            other_snakes.push(other_snake);
                        }
                    }
                    self.hits_obstacle(snake, &other_snakes, next_head, ate_food)
                }
            };
            !blocked
        });
        directions
    }

    /// 按控制模式更新指定蛇的方向。
    ///
    /// 步骤：
    /// 1. 如果是手动控制，直接跳过。
    /// 2. 如果是 AI 控制，先更新漫游状态。
    /// 3. 计算当前所有安全方向。
    /// 4. 漫游时保持当前方向，只有遇到危险才改走其他安全方向。
    /// 5. 将最终方向写回蛇实例。
    fn update_snake_direction(&mut self, slot: SnakeSlot) {
        // 先读取蛇当前的头部位置和控制状态，手动模式不参与自动转向。
        let snake = self.snake(slot);
        if !matches!(snake.controller(), SnakeController::Ai(_)) {
            return;
        }
        let head = snake.head();
        let direction = snake.direction;

        // AI 有小概率进入短暂漫游，避免始终只盯着最近食物走。
        let mut rng = rand::rng();
        if rng.random_range(0..100usize) < AI_ROAM_CHANCE_PERCENT as usize {
            self.snake_mut(slot).start_roaming(AI_ROAM_STEPS);
        }

        // 先筛出不会立刻撞墙或撞到其他蛇的安全方向。
        let mut directions = self.get_snake_safe_directions(slot);
        if directions.is_empty() {
            return;
        }

        // 漫游期间优先保持当前方向，只有当前方向不安全时才切换。
        if self.snake(slot).is_roaming() {
            if directions.contains(&direction) {
                return;
            }
            let index = rng.random_range(0..directions.len());
            self.snake_mut(slot)
                .set_direction(directions.swap_remove(index));
            return;
        }

        // 常规模式下优先选择能让蛇头更接近最近食物的安全方向。
        let direction = if let Some(target) = self
            .foods
            .iter()
            .min_by_key(|food| head.distance_to(food.point))
            .copied()
        {
            directions.sort_by_key(|direction| {
                let next_head = head.step(*direction);
                next_head.distance_to(target.point)
            });
            directions.swap_remove(0)
        } else {
            directions.swap_remove(0)
        };

        // 将最终方向写回蛇实例。
        self.snake_mut(slot).set_direction(direction);
    }

    /// 推进指定蛇的一次移动，并返回是否在碰撞中死亡。
    fn advance_snake(&mut self, slot: SnakeSlot) -> bool {
        let next_head = self.snake(slot).next_head();
        let food_index = self.foods.iter().position(|food| food.point == next_head);
        let ate_food = food_index.is_some();
        let blocked = match slot {
            SnakeSlot::Player => {
                let other_snakes = self.snakes.iter().collect::<Vec<_>>();
                self.hits_obstacle(&self.player, &other_snakes, next_head, ate_food)
            }
            SnakeSlot::Enemy(snake_index) => {
                let snake = &self.snakes[snake_index];
                let mut other_snakes = Vec::with_capacity(self.snakes.len());
                other_snakes.push(&self.player);
                for (index, other_snake) in self.snakes.iter().enumerate() {
                    if index != snake_index {
                        other_snakes.push(other_snake);
                    }
                }
                self.hits_obstacle(snake, &other_snakes, next_head, ate_food)
            }
        };
        if blocked {
            return true;
        }
        if let Some(food_index) = food_index {
            let food = self.foods.swap_remove(food_index);
            self.snake_mut(slot).eat(food);
        }
        self.snake_mut(slot).advance();
        false
    }

    /// 推进玩家蛇的一次移动。
    fn update_player(&mut self) {
        self.update_snake_direction(SnakeSlot::Player);
        if self.advance_snake(SnakeSlot::Player) {
            self.status = GameStatus::Lost;
        }
    }

    /// 推进所有 AI 蛇的一次移动。
    fn update_snakes(&mut self) {
        for snake_index in 0..self.snakes.len() {
            let slot = SnakeSlot::Enemy(snake_index);
            if !self.snakes[snake_index].is_alive() {
                if self.snakes[snake_index].tick_dead() {
                    self.spawn_snake(snake_index);
                }
                continue;
            }
            self.update_snake_direction(slot);
            if self.advance_snake(slot) {
                self.snakes[snake_index].mark_dead();
            }
        }
    }

    // render

    /// 渲染一个棋盘格子。
    fn render_cell(&self, point: Point) -> Span<'static> {
        let mut symbol = ".";
        let mut color = Color::DarkGray;
        if let Some(food) = self.foods.iter().find(|food| food.point == point) {
            symbol = food.symbol;
            color = food.color;
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
        self.spawn_food();
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
                "玩家控制: ".into(),
                self.player.controller().label().fg(self.player.head_color),
            ]),
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
        if matches!(key_event.code, KeyCode::Char('i') | KeyCode::Char('I')) {
            self.player.controller = self.player.controller.toggled();
            return;
        }
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
        if !self.player.controller().accepts_manual_input() {
            return;
        }
        if next_direction.is_opposite(self.player.direction) {
            return;
        }
        self.player.set_direction(next_direction);
    }
}
