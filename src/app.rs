use anyhow::Result;
use std::time::{Duration, Instant};

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    terminal,
};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    style::{Styled, Stylize},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Clear, Paragraph},
};

use crate::game::{
    GAMES, Game, GameKind, GameSize, GameStatus, Instruction, counter::GameCounter,
    snake::GameSnake, tetris::GameTetris,
};

const TITLE_HEIGHT: u16 = 3;
const FOOTER_HEIGHT: u16 = 3;
const STATUS_WIDTH: u16 = 24;
const GAME_BORDER_SIZE: u16 = 2;
const GAME_MIN_WIDTH: u16 = 32;
const GAME_MIN_HEIGHT: u16 = 22;
const UPDATE_INTERVAL: Duration = Duration::from_millis(33);

/// 读取当前终端尺寸并换算出游戏内容区大小。
fn current_game_size() -> Option<GameSize> {
    let Ok((width, height)) = terminal::size() else {
        return None;
    };
    calculate_game_size(width, height)
}

/// 根据终端宽高计算游戏内容区大小。
fn calculate_game_size(width: u16, height: u16) -> Option<GameSize> {
    if width < STATUS_WIDTH + GAME_BORDER_SIZE + GAME_MIN_WIDTH {
        return None;
    }
    if height < TITLE_HEIGHT + FOOTER_HEIGHT + GAME_BORDER_SIZE + GAME_MIN_HEIGHT {
        return None;
    }
    Some(GameSize {
        width: (width - STATUS_WIDTH - GAME_BORDER_SIZE) as usize,
        height: (height - TITLE_HEIGHT - FOOTER_HEIGHT - GAME_BORDER_SIZE) as usize,
    })
}

const MAIN_INSTRUCTIONS: [Instruction; 3] = [
    Instruction {
        label: " 移动 ",
        key: "<Up/Down>",
    },
    Instruction {
        label: " 进入 ",
        key: "<Enter>",
    },
    Instruction {
        label: " 退出 ",
        key: "<Q> ",
    },
];

const COMMON_INSTRUCTIONS: [Instruction; 2] = [
    Instruction {
        label: " 暂停 ",
        key: "<Space>",
    },
    Instruction {
        label: " 重开 ",
        key: "<R>",
    },
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Screen {
    #[default]
    Main,
    Game,
}

#[derive(Debug, Default)]
pub struct App {
    exit: bool,
    screen: Screen,
    game_index: usize,
    game: Option<Box<dyn Game>>,
    game_size: Option<GameSize>,
    game_status: GameStatus,
}

impl App {
    /// 创建应用，并可选择直接进入某个游戏。
    pub fn new(game: Option<GameKind>) -> Self {
        let mut app = Self::default();
        if let Some(game) = game {
            app.game_index = GAMES
                .iter()
                .position(|candidate| *candidate == game)
                .unwrap_or_default();
            app.start_game();
        }
        app
    }

    /// 运行顶层绘制与输入循环，直到用户退出程序。
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        let mut last_update = Instant::now();
        terminal.draw(|frame| self.render(frame))?;
        while !self.exit {
            let timeout = UPDATE_INTERVAL.saturating_sub(last_update.elapsed());
            let should_render = self.handle_events(timeout)?;
            let should_update = last_update.elapsed() >= UPDATE_INTERVAL;
            if should_update {
                self.update();
                last_update = Instant::now();
            }
            if should_render || should_update {
                terminal.draw(|frame| self.render(frame))?;
            }
        }
        Ok(())
    }

    /// 将整个应用标记为退出状态。
    fn exit(&mut self) {
        self.exit = true;
    }

    fn change_choose_game(&mut self, offset: isize) {
        let len = GAMES.len() as isize;
        self.game_index = ((self.game_index as isize + offset).rem_euclid(len)) as usize;
    }

    /// 从当前游戏返回主界面。
    fn return_to_main(&mut self) {
        self.game_index = 0;
        self.game = None;
        self.game_size = None;
        self.game_status = GameStatus::Main;
        self.screen = Screen::Main;
    }

    /// 打开所选游戏，并重置应用层的游戏状态。
    fn start_game(&mut self) {
        let game = GAMES[self.game_index];
        let game_size = current_game_size();
        let size = game_size.unwrap_or_default();
        self.game = Some(match game {
            GameKind::Counter => Box::new(GameCounter::new(size)),
            GameKind::Snake => Box::new(GameSnake::new(size)),
            GameKind::Tetris => Box::new(GameTetris::new(size)),
        });
        self.game_size = game_size;
        self.game_status = if game_size.is_some() {
            GameStatus::Ready
        } else {
            GameStatus::WindowTooSmall
        };
        self.sync_game_status();
        self.screen = Screen::Game;
    }

    /// 切换当前游戏界面的应用层暂停状态。
    fn pause_game(&mut self) {
        if matches!(self.game_status, GameStatus::Running) {
            self.game_status = GameStatus::Paused;
            return;
        }
        if matches!(self.game_status, GameStatus::Paused) {
            self.game_status = GameStatus::Running;
            return;
        }
    }

    /// 更新应用逻辑。
    fn update(&mut self) {
        if self.screen != Screen::Game {
            return;
        }
        if self.game_status != GameStatus::Running {
            return;
        }
        if let Some(game) = self.game.as_mut() {
            game.update();
        }
        self.sync_game_status();
    }

    /// 从当前游戏同步应用层关心的特殊状态。
    fn sync_game_status(&mut self) {
        let Some(game) = self.game.as_ref() else {
            return;
        };
        match game.status() {
            GameStatus::Won | GameStatus::Lost | GameStatus::WindowTooSmall => {
                self.game_status = game.status();
            }
            _ => {}
        }
    }

    /// 根据当前界面分发渲染逻辑。
    fn render(&self, frame: &mut Frame) {
        match self.screen {
            Screen::Main => self.render_main(frame),
            Screen::Game => self.render_game(frame),
        }
        if matches!(
            self.game_status,
            GameStatus::Ready
                | GameStatus::Paused
                | GameStatus::Won
                | GameStatus::Lost
                | GameStatus::WindowTooSmall
        ) {
            // 获取弹窗区域
            let [_, popup_area, _] = Layout::vertical([
                Constraint::Fill(1),
                Constraint::Length(7),
                Constraint::Fill(1),
            ])
            .areas(frame.area());
            let [_, popup_area, _] = Layout::horizontal([
                Constraint::Fill(1),
                Constraint::Length(32),
                Constraint::Fill(1),
            ])
            .areas(popup_area);
            // 清除弹窗区域
            frame.render_widget(Clear, popup_area);
            // 渲染弹窗
            frame.render_widget(self.render_popup(), popup_area);
        }
    }

    /// 渲染游戏选择界面。
    fn render_main(&self, frame: &mut Frame) {
        let [title_area, content_area, footer_area] = Layout::vertical([
            Constraint::Length(TITLE_HEIGHT),
            Constraint::Min(0),
            Constraint::Length(FOOTER_HEIGHT),
        ])
        .areas(frame.area());
        frame.render_widget(self.render_title(), title_area);
        frame.render_widget(self.render_main_content(), content_area);
        frame.render_widget(self.render_footer(), footer_area);
    }

    /// 渲染当前游戏界面，包括内容区和状态区。
    fn render_game(&self, frame: &mut Frame) {
        let [title_area, content_area, footer_area] = Layout::vertical([
            Constraint::Length(TITLE_HEIGHT),
            Constraint::Min(0),
            Constraint::Length(FOOTER_HEIGHT),
        ])
        .areas(frame.area());
        let [content_area, status_area] =
            Layout::horizontal([Constraint::Min(0), Constraint::Length(STATUS_WIDTH)])
                .areas(content_area);
        frame.render_widget(self.render_title(), title_area);
        frame.render_widget(self.render_game_content(), content_area);
        frame.render_widget(self.render_game_status(), status_area);
        frame.render_widget(self.render_footer(), footer_area);
    }

    /// 渲染当前界面的标题区域。
    fn render_title(&self) -> Paragraph<'static> {
        let title = match self.screen {
            Screen::Main => "选择游戏".to_string(),
            Screen::Game => GAMES[self.game_index].name().to_string(),
        };
        Paragraph::new(title.bold())
            .centered()
            .block(Block::bordered().title("Title").border_set(border::THICK))
    }

    /// 渲染主界面上的可选游戏列表。
    fn render_main_content(&self) -> Paragraph<'static> {
        let mut lines = vec![];
        lines.extend(GAMES.iter().enumerate().map(|(index, game)| {
            let game_name = game.name();
            if index == self.game_index {
                Line::from(format!("> {game_name}")).yellow()
            } else {
                Line::from(format!("  {game_name}"))
            }
        }));
        Paragraph::new(Text::from(lines))
            .centered()
            .block(Block::bordered().title("Main").border_set(border::THICK))
    }

    /// 渲染当前游戏的内容区域。
    fn render_game_content(&self) -> Paragraph<'static> {
        let text = self
            .game
            .as_ref()
            .map(|game| game.render_content())
            .unwrap_or_else(|| Text::from("No Game"));
        Paragraph::new(text)
            .centered()
            .block(Block::bordered().title("Game").border_set(border::THICK))
    }

    /// 渲染游戏状态以及应用层的暂停信息。
    fn render_game_status(&self) -> Paragraph<'static> {
        let mut text = self
            .game
            .as_ref()
            .map(|game| game.render_status())
            .unwrap_or_else(|| Text::from("No Status"));
        text.lines.push(Line::from(vec![
            "状态: ".into(),
            self.game_status.label().set_style(self.game_status.style()),
        ]));
        Paragraph::new(text).block(Block::bordered().title("Status").border_set(border::THICK))
    }

    /// 渲染当前界面底部的帮助提示行。
    fn render_footer(&self) -> Paragraph<'static> {
        let instructions = match self.screen {
            Screen::Main => MAIN_INSTRUCTIONS.to_vec(),
            Screen::Game => {
                let mut instructions = self
                    .game
                    .as_ref()
                    .map(|game| game.instructions())
                    .unwrap_or_default();
                instructions.extend_from_slice(&COMMON_INSTRUCTIONS);
                instructions
            }
        };
        let mut spans = Vec::with_capacity(instructions.len() * 2);
        for instruction in &instructions {
            spans.push(instruction.label.into());
            spans.push(instruction.key.blue().bold());
        }
        Paragraph::new(Text::from(vec![Line::from(spans)]))
            .centered()
            .block(Block::bordered().title("Help").border_set(border::THICK))
    }

    /// 根据当前应用状态渲染居中的弹窗。
    fn render_popup(&self) -> Paragraph<'static> {
        let (title, lines) = match self.game_status {
            GameStatus::Idle => unreachable!(),
            GameStatus::Main => unreachable!(),
            GameStatus::Running => unreachable!(),
            GameStatus::Ready => (
                "准备",
                vec![
                    Line::from("游戏已准备就绪").centered(),
                    Line::from(""),
                    Line::from("按任意键开始").centered(),
                    Line::from("按 Q 返回主界面").centered(),
                ],
            ),
            GameStatus::Paused => (
                "暂停",
                vec![
                    Line::from("游戏已暂停").centered(),
                    Line::from(""),
                    Line::from("按 Space 继续").centered(),
                    Line::from("按 R 重新开始").centered(),
                    Line::from("按 Q 返回主界面").centered(),
                ],
            ),
            GameStatus::Won => (
                "胜利",
                vec![
                    Line::from("恭喜，游戏胜利").centered(),
                    Line::from(""),
                    Line::from("按 R 再来一局").centered(),
                    Line::from("按 Q 返回主界面").centered(),
                    Line::from(""),
                ],
            ),
            GameStatus::Lost => (
                "失败",
                vec![
                    Line::from("这局失败了").centered(),
                    Line::from(""),
                    Line::from("按 R 重新开始").centered(),
                    Line::from("按 Q 返回主界面").centered(),
                    Line::from(""),
                ],
            ),
            GameStatus::WindowTooSmall => (
                "窗口太小",
                vec![
                    Line::from("当前窗口太小，无法正常显示游戏").centered(),
                    Line::from(""),
                    Line::from("请放大终端窗口").centered(),
                    Line::from("调整后会自动重新开始").centered(),
                    Line::from("按 Q 返回主界面").centered(),
                ],
            ),
        };
        Paragraph::new(Text::from(lines).centered()).block(
            Block::bordered()
                .title(Line::from(title).set_style(self.game_status.style()).bold())
                .border_set(border::THICK)
                .border_style(self.game_status.style()),
        )
    }

    /// 读取终端事件，并将支持的按键事件转发给应用。
    fn handle_events(&mut self, timeout: Duration) -> Result<bool> {
        if !event::poll(timeout)? {
            return Ok(false);
        }
        match event::read()? {
            // 这里必须确认事件是按键按下事件，
            // 因为 crossterm 在 Windows 上还会发出按键释放和重复事件。
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event);
                Ok(true)
            }
            Event::Resize(width, height) => {
                self.handle_resize(width, height);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// 在终端尺寸变化后决定是否重开当前游戏。
    fn handle_resize(&mut self, width: u16, height: u16) {
        if self.screen != Screen::Game {
            return;
        }
        let game_size = calculate_game_size(width, height);
        if self.game_size == game_size {
            return;
        }
        self.start_game();
    }

    /// 将按键事件分发给当前界面对应的处理函数。
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match self.screen {
            Screen::Main => self.handle_main_keys(key_event),
            Screen::Game => self.handle_game_keys(key_event),
        }
    }

    /// 处理主界面的导航按键。
    fn handle_main_keys(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Up => self.change_choose_game(-1),
            KeyCode::Down => self.change_choose_game(1),
            KeyCode::Enter => self.start_game(),
            _ => {}
        }
    }

    /// 处理游戏界面的应用层控制，并转发游戏输入。
    fn handle_game_keys(&mut self, key_event: KeyEvent) {
        if matches!(key_event.code, KeyCode::Char('q') | KeyCode::Esc) {
            self.return_to_main();
            return;
        }
        if self.game_status == GameStatus::Ready {
            self.game_status = GameStatus::Running;
            return;
        }
        if matches!(key_event.code, KeyCode::Char('r')) {
            self.start_game();
            return;
        }
        if matches!(key_event.code, KeyCode::Char(' ')) {
            self.pause_game();
            return;
        }
        if self.game_status != GameStatus::Running {
            return;
        }
        if let Some(game) = self.game.as_mut() {
            game.handle_key_event(key_event);
        }
        self.sync_game_status();
    }
}
