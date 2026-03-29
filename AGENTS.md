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
  main.rs
  app.rs
  game/
    mod.rs
    counter.rs
    snake.rs
```

## 约定

- 游戏内部尺寸、计数、索引优先使用 `usize`
- 游戏内部坐标优先使用 `isize`
- 只在和 `ratatui` / `crossterm` 交界时保留 `u16`
- 新增游戏时优先复用 `game/mod.rs` 中的 `Game` trait
- 修改功能时同步更新 `README.md`、`ARCHITECTURE.md`、`TODO.md`
- 只有在状态真实变化后才更新文档，避免文档先于实现

## 当前重点文件

- `src/app.rs`
  负责顶层循环、界面切换、统一弹窗、尺寸变化和应用层状态
- `src/game/mod.rs`
  放共享类型、`GameKind` 和 `Game` trait
- `src/game/counter.rs`
  最小示例游戏
- `src/game/snake.rs`
  复杂度最高的实验场，包含 AI、超级食物、尸块转食物和风险空间判断

## 常用命令

- `cargo run`
- `cargo run -- --game counter`
- `cargo run -- --game snake`
- `cargo test`

## 修改建议

- 如果改了 `snake.rs` 的状态结构，优先检查 `update_symbols()` 是否仍然覆盖了所有符号来源
- 如果改了尺寸或坐标类型，优先检查 `app.rs` 和 `ratatui` 边界转换
- 如果新增游戏状态，记得同步 `App` 的弹窗和状态面板
