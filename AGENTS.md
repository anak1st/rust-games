# AGENTS

## 项目定位

这是一个用 Rust 和 `ratatui` 编写的小型 TUI 游戏实验项目。

当前目标不是做完整游戏框架，而是围绕一个尽量小的代码库，持续验证这些事情：

- 多界面切换
- `App` 外壳和具体游戏之间的职责边界
- 固定 tick 的逻辑更新
- `ratatui` 渲染组织方式
- 新游戏接入成本

## 仓库结构

```text
src/
  main.rs          # 程序入口，命令行参数解析
  app.rs           # 顶层循环、界面切换、统一弹窗、尺寸变化
  game/
    mod.rs         # 共享类型、GameKind、Game trait、RenderBuffer
    counter.rs     # 最小示例游戏
    snake.rs       # 复杂度最高的实验场
```

## 常用命令

```bash
# 运行
cargo run                    # 默认运行
cargo run -- --game counter  # 运行指定游戏
cargo run -- --game snake    # 运行贪吃蛇

# 测试
cargo test                   # 运行所有测试
cargo test -- <pattern>      # 运行匹配模式的测试
cargo test counter           # 运行特定游戏的测试
cargo test snake             # 运行贪吃蛇的测试

# 代码质量
cargo fmt                    # 格式化代码
cargo clippy                 # 运行 clippy 检查
cargo clippy -- -D warnings  # clippy 严格模式
cargo check                  # 快速类型检查
```

## 类型约定

- **游戏内部**：尺寸、计数、索引优先使用 `usize`
- **游戏内部坐标**：优先使用 `isize`
- **与 ratatui/crossterm 交界**：保留 `u16`
- 边界转换时使用 `as` 显式转换

```rust
// Good: 游戏内部用 usize/isize
struct GameSize { width: usize, height: usize }
struct Point { x: isize, y: isize }

// Good: 与库交界时用 u16
fn handle_resize(&mut self, width: u16, height: u16) {
    let game_size = calculate_game_size(width, height);
}

// Bad: 不要在游戏内部使用 u16 作为尺寸
```

## 命名约定

- **类型名、枚举名、枚举变体**：PascalCase（`GameStatus`, `SnakeController`）
- **函数、方法、变量**：snake_case（`update_symbols`, `game_size`）
- **常量**：SCREAMING_SNAKE_CASE（`UPDATE_INTERVAL`, `FOOD_COUNT`）
- **公开 trait 方法**：简短动词（`update`, `status`, `render_content`）
- **公开 getter**：直接使用字段名或 `xxx()` 形式

## 导入组织

按以下顺序分组，组内按字母排序：

```rust
// 1. 标准库
use std::time::{Duration, Instant};

// 2. 外部 crate（按字母顺序）
use anyhow::Result;
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{DefaultTerminal, Frame, ...};

// 3. 本地 crate（相对路径）
use crate::game::{
    GAMES, Game, GameKind, GameSize, GameStatus, Instruction,
    counter::GameCounter,
    snake::GameSnake,
};
```

## 错误处理

使用 `anyhow::Result<()>` 作为顶层返回类型：

```rust
// main.rs 和 app.rs 使用这种模式
fn main() -> Result<()> {
    let args = Args::parse();
    let mut app = app::App::new(args.game);
    ratatui::run(|terminal| app.run(terminal))?;
    Ok(())
}
```

内部使用 `?` 操作符和 `anyhow::anyhow!`：

```rust
fn current_game_size() -> Option<GameSize> {
    let Ok((width, height)) = terminal::size() else {
        return None;
    };
    calculate_game_size(width, height)
}
```

游戏内部状态不使用 Result，而是使用 `GameStatus` 枚举。

## 代码风格

### 函数设计

- 公开函数使用中文 doc comment
- 函数长度尽量控制在 50 行以内
- 早期返回优先于深层嵌套

```rust
/// 创建应用，并可选择直接进入某个游戏。
pub fn new(game: Option<GameKind>) -> Self {
    let mut app = Self::default();
    if let Some(game) = game {
        app.game_index = GAMES.iter().position(|c| *c == game).unwrap_or_default();
        app.start_game();
    }
    app
}
```

### 匹配表达式

使用 `matches!` 宏进行简单匹配：

```rust
// Good
if matches!(self.game_status, GameStatus::Running) {
    self.game_status = GameStatus::Paused;
    return;
}

// Good: match 用于需要返回值
fn status(&self) -> GameStatus {
    match self {
        GameStatus::Idle => "空闲",
        // ...
    }
}
```

### Option/Result 处理

优先使用 `if let ... else { return }` 或 `let Some(...) = ... else { return }` 模式：

```rust
// Good
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
```

### 结构体构造

使用构造者风格（field: value）或直接初始化：

```rust
// Good: 显式命名
let game = Self {
    size,
    status: GameStatus::Running,
    player: Self::spawn_player(size),
    snakes: vec![],
    // ...
};

// Good: 常量作为默认值
const INSTRUCTIONS: [Instruction; 2] = [
    Instruction { label: " 移动 ", key: "<Up/Down>" },
    // ...
];
```

### 迭代器使用

适度使用迭代器，保持可读性：

```rust
// Good
let game_name = GAMES
    .iter()
    .position(|candidate| *candidate == game)
    .unwrap_or_default();

// Good: for 循环当迭代器降低可读性时
for direction in [Direction::Up, Direction::Down, Direction::Left, Direction::Right] {
    // ...
}
```

## 游戏接入流程

1. 在 `src/game/mod.rs` 中添加 `GameKind` 枚举变体
2. 实现 `Game` trait
3. 在 `App::start_game()` 中添加构造逻辑
4. 更新 `GAMES` 数组

## 修改建议

- 如果改了 `snake.rs` 的状态结构，优先检查 `update_symbols()` 是否仍然覆盖了所有符号来源
- 如果改了尺寸或坐标类型，优先检查 `app.rs` 和 `ratatui` 边界转换
- 如果新增游戏状态，记得同步 `App` 的弹窗和状态面板
- 修改功能时同步更新 `README.md`、`ARCHITECTURE.md`、`TODO.md`
- 只有在状态真实变化后才更新文档，避免文档先于实现

## 框架依赖

```toml
anyhow = "1.0.102"      # 错误处理
clap = "4.6.0"          # 命令行参数解析
crossterm = "0.29.0"    # 终端事件
rand = "0.10.0"         # 随机数
ratatui = "0.30.0"      # TUI 渲染
```
