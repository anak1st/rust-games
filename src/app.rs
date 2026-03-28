use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph},
};

use crate::game::{GAMES, Game, GameKind, Instruction, counter::CounterGame};

const MAIN_INSTRUCTIONS: [Instruction; 3] = [
    Instruction {
        label: " Move ",
        key: "<Up/Down>",
    },
    Instruction {
        label: " Enter ",
        key: "<Enter>",
    },
    Instruction {
        label: " Quit ",
        key: "<Q> ",
    },
];

const COMMON_INSTRUCTIONS: [Instruction; 2] = [
    Instruction {
        label: " Pause ",
        key: "<P>",
    },
    Instruction {
        label: " Restart ",
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
    index: usize,
    paused: bool,
    game: Option<Box<dyn Game>>,
    screen: Screen,
}

impl App {
    /// 创建应用，并可选择直接进入某个游戏。
    pub fn new(game: Option<GameKind>) -> Self {
        let mut app = Self::default();
        if let Some(game) = game {
            app.open_game(game);
        }
        app
    }

    /// 运行顶层绘制与输入循环，直到用户退出程序。
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    /// 打开所选游戏，并重置应用层的游戏状态。
    fn open_game(&mut self, game: GameKind) {
        match game {
            GameKind::Counter => {
                self.index = 0;
                self.paused = false;
                self.game = Some(Box::new(CounterGame::default()));
                self.screen = Screen::Game;
            }
        }
    }

    /// 从当前游戏返回主界面。
    fn return_to_main(&mut self) {
        self.index = 0;
        self.paused = false;
        self.game = None;
        self.screen = Screen::Main;
    }

    /// 将整个应用标记为退出状态。
    fn exit(&mut self) {
        self.exit = true;
    }

    /// 切换当前游戏界面的应用层暂停状态。
    fn pause(&mut self) {
        self.paused = !self.paused;
    }

    /// 根据当前界面分发绘制逻辑。
    fn draw(&self, frame: &mut Frame) {
        match self.screen {
            Screen::Main => self.draw_main(frame),
            Screen::Game => self.draw_game(frame),
        }
    }

    /// 绘制游戏选择界面。
    fn draw_main(&self, frame: &mut Frame) {
        let [title_area, content_area, footer_area] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .areas(frame.area());
        frame.render_widget(self.render_title(), title_area);
        frame.render_widget(self.render_main_content(), content_area);
        frame.render_widget(self.render_footer(), footer_area);
    }

    /// 绘制当前游戏界面，包括内容区和状态区。
    fn draw_game(&self, frame: &mut Frame) {
        let [title_area, content_area, footer_area] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .areas(frame.area());
        let [content_area, status_area] =
            Layout::horizontal([Constraint::Min(0), Constraint::Length(24)]).areas(content_area);
        frame.render_widget(self.render_title(), title_area);
        frame.render_widget(self.render_game_content(), content_area);
        frame.render_widget(self.render_game_status(), status_area);
        frame.render_widget(self.render_footer(), footer_area);
    }

    /// 渲染当前界面的标题区域。
    fn render_title(&self) -> Paragraph<'static> {
        let title = match self.screen {
            Screen::Main => "选择游戏".to_string(),
            Screen::Game => self
                .game
                .as_ref()
                .map(|game| game.title())
                .unwrap_or_else(|| "Unknown Game".to_string()),
        };
        Paragraph::new(title.bold())
            .centered()
            .block(Block::bordered().title("Title").border_set(border::THICK))
    }

    /// 渲染主界面上的可选游戏列表。
    fn render_main_content(&self) -> Paragraph<'static> {
        let mut lines = vec![];
        lines.extend(GAMES.iter().enumerate().map(|(idx, game)| {
            let game_name = match game {
                GameKind::Counter => "Counter Demo",
            };
            if idx == self.index {
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
            .map(|game| game.content())
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
            .map(|game| game.status())
            .unwrap_or_else(|| Text::from("No Status"));
        text.lines.push(Line::from(vec![
            "Paused: ".into(),
            if self.paused {
                "yes".yellow()
            } else {
                "no".green()
            },
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

    /// 读取终端事件，并将支持的按键事件转发给应用。
    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            // 这里必须确认事件是按键按下事件，
            // 因为 crossterm 在 Windows 上还会发出按键释放和重复事件。
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
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
            KeyCode::Up => self.index = (self.index + 1) % GAMES.len(),
            KeyCode::Down => self.index = (self.index + GAMES.len() - 1) % GAMES.len(),
            KeyCode::Enter => self.open_game(GAMES[self.index]),
            _ => {}
        }
    }

    /// 处理游戏界面的应用层控制，并转发游戏输入。
    fn handle_game_keys(&mut self, key_event: KeyEvent) {
        if matches!(key_event.code, KeyCode::Char('q') | KeyCode::Esc) {
            self.return_to_main();
            return;
        }
        if matches!(key_event.code, KeyCode::Char('p')) {
            self.pause();
            return;
        }
        if self.paused {
            return;
        }
        if let Some(game) = self.game.as_mut() {
            game.handle_key_event(key_event);
        }
    }
}
