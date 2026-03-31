# AGENTS

## 项目定位

Rust + `ratatui` TUI 游戏实验项目，验证多界面切换、App/Game 职责边界、固定 tick 更新、ratatui 渲染组织、新游戏接入成本。

## 仓库结构

```text
src/
  main.rs          # 入口，命令行参数解析
  app.rs           # 顶层循环、界面切换、弹窗、尺寸变化
  game/
    mod.rs         # 共享类型、GameKind、Game trait、RenderBuffer
    counter.rs     # 最小示例游戏
    snake.rs       # 玩家蛇 + AI 敌蛇 + 超级食物 + 尸块转食物
    tetris.rs      # 固定 10x20 棋盘俄罗斯方块
```

## 常用命令

```bash
# 运行
cargo run                    # 默认运行
cargo run -- --game counter  # 指定游戏
cargo run -- --game snake
cargo run -- --game tetris

# 测试
cargo test                   # 所有测试
cargo test -- <pattern>      # 匹配模式
cargo test snake             # 特定游戏

# 代码质量
cargo fmt                    # 格式化
cargo check                  # 快速类型检查
cargo clippy                 # lint
cargo clippy -- -D warnings  # 严格模式（CI 用）
```

## 类型约定

- **游戏内部尺寸/计数/索引**：`usize`
- **游戏内部坐标**：`isize`
- **与 ratatui/crossterm 交界**：保留 `u16`
- 边界转换用 `as` 显式转换

```rust
struct GameSize { width: usize, height: usize }  // Good
struct Point { x: isize, y: isize }               // Good
fn handle_resize(&mut self, width: u16, height: u16) { ... }  // Good: 库边界
```

## 命名约定

- **类型/枚举/枚举变体**：PascalCase（`GameStatus`, `SnakeController`）
- **函数/方法/变量**：snake_case（`update_symbols`, `game_size`）
- **常量**：SCREAMING_SNAKE_CASE（`UPDATE_INTERVAL`, `FOOD_COUNT`）
- **trait 方法**：简短动词（`update`, `status`, `render_content`）

## 导入组织

按标准库 → 外部 crate（字母序）→ 本地 crate 分组，组内字母排序：

```rust
use std::time::{Duration, Instant};

use anyhow::Result;
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{DefaultTerminal, Frame, ...};

use crate::game::{GAMES, Game, GameKind, GameSize, GameStatus, ...};
```

## 错误处理

顶层用 `anyhow::Result<()>`；内部用 `?` 和 `anyhow::anyhow!`；游戏状态用 `GameStatus` 枚举而非 Result。

```rust
fn main() -> Result<()> { ... }  // 顶层
let Some(game) = self.game.as_ref() else { return; };  // 内部早返回
```

## 代码风格

- 公开函数用中文 doc comment
- 函数控制在 50 行内，优先早期返回
- 简单匹配用 `matches!` 宏，需要返回值时用 `match`
- Option/Result 优先 `let Some(x) = ... else { return }` 模式
- 结构体用显式命名字段初始化
- 迭代器适度使用，可读性下降时换 for 循环

```rust
if matches!(self.game_status, GameStatus::Running) {
    self.game_status = GameStatus::Paused;
    return;
}
```

## 游戏接入流程

1. `src/game/mod.rs` 添加 `GameKind` 变体
2. 实现 `Game` trait（`update`, `status`, `render_content`, `render_status`, `instructions`, `handle_key_event`）
3. `App::start_game()` 添加构造分支
4. 更新 `GAMES` 数组

## 修改建议

- 改 `snake.rs` 状态结构后，检查 `update_symbols()` 覆盖所有符号来源
- 改尺寸/坐标类型后，检查 `app.rs` 与 ratatui 边界转换
- 新增游戏状态时，同步 App 的弹窗和状态面板
- 功能变更同步更新 `README.md`、`ARCHITECTURE.md`、`TODO.md`

## 框架依赖

```toml
anyhow = "1.0.102"      # 错误处理
clap = "4.6.0"          # 命令行参数（derive）
crossterm = "0.29.0"    # 终端事件
rand = "0.10.0"         # 随机数
ratatui = "0.30.0"      # TUI 渲染
edition = "2024"
```
