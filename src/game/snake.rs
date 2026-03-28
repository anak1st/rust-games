use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    style::Stylize,
    text::{Line, Text},
};

use crate::game::{Game, GameSize, GameStatus, Instruction};

const INSTRUCTIONS: [Instruction; 1] = [Instruction {
    label: " 转向 ",
    key: "<Arrows/WASD>",
}];

const MIN_WIDTH: u16 = 12;
const MIN_HEIGHT: u16 = 8;
const FRAMES_PER_STEP: u8 = 4;

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
    fn label(self) -> &'static str {
        match self {
            Direction::Up => "上",
            Direction::Down => "下",
            Direction::Left => "左",
            Direction::Right => "右",
        }
    }

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
pub struct GameSnake {
    size: GameSize,
    status: GameStatus,
    snake: Vec<Point>,
    direction: Direction,
    food: Point,
    score: usize,
    frame: u8,
}

impl GameSnake {
    pub fn new(size: GameSize) -> Self {
        if size.width < MIN_WIDTH || size.height < MIN_HEIGHT {
            return Self {
                size,
                status: GameStatus::WindowTooSmall,
                snake: vec![],
                direction: Direction::Right,
                food: Point::default(),
                score: 0,
                frame: 0,
            };
        }
        let center_x = (size.width / 2) as i16;
        let center_y = (size.height / 2) as i16;
        let snake = vec![
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
        ];
        let food = Self::next_food(&snake, size, 0).unwrap_or_default();
        Self {
            size,
            status: GameStatus::Running,
            snake,
            direction: Direction::Right,
            food,
            score: 0,
            frame: 0,
        }
    }

    fn next_head(&self) -> Point {
        let head = self.snake[0];
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

    fn next_food(snake: &[Point], size: GameSize, score: usize) -> Option<Point> {
        let width = size.width as usize;
        let height = size.height as usize;
        let cell_count = width * height;
        let start = (score * 7 + snake.len()) % cell_count;
        for offset in 0..cell_count {
            let index = (start + offset) % cell_count;
            let point = Point {
                x: (index % width) as i16,
                y: (index / width) as i16,
            };
            if !snake.contains(&point) {
                return Some(point);
            }
        }
        None
    }
}

impl Game for GameSnake {
    fn update(&mut self) {
        self.frame += 1;
        if self.frame < FRAMES_PER_STEP {
            return;
        }
        self.frame = 0;
        let next_head = self.next_head();
        if next_head.x < 0
            || next_head.y < 0
            || next_head.x >= self.size.width as i16
            || next_head.y >= self.size.height as i16
        {
            self.status = GameStatus::Lost;
            return;
        }
        let ate_food = next_head == self.food;
        let body_len = if ate_food {
            self.snake.len()
        } else {
            self.snake.len() - 1
        };
        if self.snake[..body_len].contains(&next_head) {
            self.status = GameStatus::Lost;
            return;
        }
        self.snake.insert(0, next_head);
        if ate_food {
            self.score += 1;
            if let Some(food) = Self::next_food(&self.snake, self.size, self.score) {
                self.food = food;
            } else {
                self.status = GameStatus::Won;
            }
            return;
        }
        self.snake.pop();
    }

    fn status(&self) -> GameStatus {
        self.status
    }

    fn render_content(&self) -> Text<'static> {
        if self.status == GameStatus::WindowTooSmall {
            return Text::from("贪吃蛇区域太小");
        }
        let mut lines = Vec::with_capacity(self.size.height as usize);
        for y in 0..self.size.height as i16 {
            let mut row = String::with_capacity(self.size.width as usize);
            for x in 0..self.size.width as i16 {
                let point = Point { x, y };
                let cell = if self.snake.first() == Some(&point) {
                    '@'
                } else if self.food == point {
                    '*'
                } else if self.snake[1..].contains(&point) {
                    'o'
                } else {
                    ' '
                };
                row.push(cell);
            }
            lines.push(Line::from(row));
        }
        Text::from(lines)
    }

    fn render_status(&self) -> Text<'static> {
        Text::from(vec![
            Line::from(vec!["分数: ".into(), self.score.to_string().yellow()]),
            Line::from(vec!["长度: ".into(), self.snake.len().to_string().yellow()]),
            Line::from(vec!["方向: ".into(), self.direction.label().yellow()]),
            Line::from(vec![
                "尺寸: ".into(),
                format!("{} x {}", self.size.width, self.size.height).yellow(),
            ]),
            Line::from(vec![
                "速度: ".into(),
                format!("每 {} 帧移动 1 格", FRAMES_PER_STEP).yellow(),
            ]),
        ])
    }

    fn instructions(&self) -> Vec<Instruction> {
        INSTRUCTIONS.to_vec()
    }

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
        if next_direction.is_opposite(self.direction) {
            return;
        }
        self.direction = next_direction;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_returns_window_too_small_for_small_board() {
        let game = GameSnake::new(GameSize {
            width: 8,
            height: 6,
        });

        assert_eq!(game.status(), GameStatus::WindowTooSmall);
    }

    #[test]
    fn update_eats_food_and_grows() {
        let mut game = GameSnake {
            size: GameSize {
                width: 12,
                height: 8,
            },
            status: GameStatus::Running,
            snake: vec![
                Point { x: 3, y: 3 },
                Point { x: 2, y: 3 },
                Point { x: 1, y: 3 },
            ],
            direction: Direction::Right,
            food: Point { x: 4, y: 3 },
            score: 0,
            frame: FRAMES_PER_STEP - 1,
        };

        game.update();

        assert_eq!(game.score, 1);
        assert_eq!(game.snake.len(), 4);
        assert_eq!(game.snake[0], Point { x: 4, y: 3 });
        assert_eq!(game.status(), GameStatus::Running);
    }

    #[test]
    fn update_hits_wall_and_loses() {
        let mut game = GameSnake {
            size: GameSize {
                width: 12,
                height: 8,
            },
            status: GameStatus::Running,
            snake: vec![
                Point { x: 11, y: 3 },
                Point { x: 10, y: 3 },
                Point { x: 9, y: 3 },
            ],
            direction: Direction::Right,
            food: Point { x: 0, y: 0 },
            score: 0,
            frame: FRAMES_PER_STEP - 1,
        };

        game.update();

        assert_eq!(game.status(), GameStatus::Lost);
    }
}
