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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct Point {
    x: i16,
    y: i16,
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
}

#[derive(Debug)]
struct Snake {
    body: Vec<Point>,
    direction: Direction,
    head_symbol: &'static str,
    body_symbol: &'static str,
    head_color: Color,
    body_color: Color,
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

    /// 根据当前方向计算下一帧蛇头的位置。
    fn next_head(&self) -> Point {
        let head = self.head();
        match self.direction {
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

    /// 判断给定位置是否被整条蛇占用。
    fn occupies(&self, point: Point) -> bool {
        self.body.contains(&point)
    }

    /// 判断给定位置是否被蛇身占用。
    fn body_contains(&self, point: Point) -> bool {
        self.body[1..].contains(&point)
    }

    /// 将蛇移动到新的头部位置，并按需决定是否增长。
    fn move_to(&mut self, next_head: Point, grow: bool) {
        self.body.insert(0, next_head);
        if !grow {
            self.body.pop();
        }
    }

    /// 更新蛇当前的移动方向。
    fn set_direction(&mut self, direction: Direction) {
        self.direction = direction;
    }
}

#[derive(Debug)]
pub struct GameSnake {
    size: GameSize,
    status: GameStatus,
    snake: Snake,
    foods: Vec<Point>,
    score: usize,
    frame: u8,
}

impl GameSnake {
    /// 创建一个新的贪吃蛇游戏实例。
    pub fn new(size: GameSize) -> Self {
        if size.width < MIN_WIDTH || size.height < MIN_HEIGHT {
            return Self {
                size,
                status: GameStatus::WindowTooSmall,
                snake: Snake::new_player(vec![]),
                foods: vec![],
                score: 0,
                frame: 0,
            };
        }
        let center_x = (size.width / 2) as i16;
        let center_y = (size.height / 2) as i16;
        let snake = Snake::new_player(vec![
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
            snake,
            foods: vec![],
            score: 0,
            frame: 0,
        };
        game.fill_foods();
        game
    }

    /// 为棋盘补齐缺少的食物数量。
    fn fill_foods(&mut self) {
        let mut empty_points = Vec::new();
        for y in 0..self.size.height as i16 {
            for x in 0..self.size.width as i16 {
                let point = Point { x, y };
                if self.snake.occupies(point) || self.foods.contains(&point) {
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
        let next_head = self.snake.next_head();
        if next_head.x < 0
            || next_head.y < 0
            || next_head.x >= self.size.width as i16
            || next_head.y >= self.size.height as i16
        {
            self.status = GameStatus::Lost;
            return;
        }
        let food_index = self.foods.iter().position(|food| *food == next_head);
        let ate_food = food_index.is_some();
        let body_len = if ate_food {
            self.snake.len()
        } else {
            self.snake.len() - 1
        };
        if self.snake.body[..body_len].contains(&next_head) {
            self.status = GameStatus::Lost;
            return;
        }
        self.snake.move_to(next_head, ate_food);
        if ate_food {
            self.score += 1;
            if let Some(food_index) = food_index {
                self.foods.swap_remove(food_index);
            }
            self.fill_foods();
            if self.foods.is_empty() {
                self.status = GameStatus::Won;
            }
            return;
        }
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
                let point = Point { x, y };
                let span = if self.snake.head() == point {
                    Span::styled(
                        self.snake.head_symbol,
                        Style::new().fg(self.snake.head_color),
                    )
                } else if self.foods.contains(&point) {
                    Span::styled("*", Style::new().fg(Color::Green))
                } else if self.snake.body_contains(point) {
                    Span::styled(
                        self.snake.body_symbol,
                        Style::new().fg(self.snake.body_color),
                    )
                } else {
                    Span::styled(".", Style::new().fg(Color::DarkGray))
                };
                spans.push(span);
            }
            lines.push(Line::from(spans));
        }
        Text::from(lines)
    }

    /// 渲染贪吃蛇游戏的状态区域。
    fn render_status(&self) -> Text<'static> {
        Text::from(vec![
            Line::from(vec![
                "分数: ".into(),
                self.score.to_string().fg(self.snake.head_color),
            ]),
            Line::from(vec![
                "长度: ".into(),
                self.snake.len().to_string().fg(self.snake.head_color),
            ]),
            Line::from(vec!["食物: ".into(), self.foods.len().to_string().green()]),
            Line::from(vec![
                "方向: ".into(),
                self.snake.direction.label().fg(self.snake.head_color),
            ]),
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
        if next_direction.is_opposite(self.snake.direction) {
            return;
        }
        self.snake.set_direction(next_direction);
    }
}
