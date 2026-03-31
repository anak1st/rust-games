# rust-games

一个用 Rust 编写的 `ratatui` 终端游戏练习项目。

当前仓库包含三个游戏：

- 计数器
- 贪吃蛇
- 俄罗斯方块

其中贪吃蛇已经包含这些机制：

- 玩家手动控制
- AI 敌蛇
- 超级食物
- 尸块延时转食物
- 基于局部空间的风险判断
- 符号表缓存

## 运行

```bash
cargo run
```

直接进入指定游戏：

```bash
cargo run -- --game counter
cargo run -- --game snake
cargo run -- --game tetris
```

## 控制

主界面：

- `Up/Down` 选择游戏
- `Enter` 进入游戏
- `q` 退出

计数器：

- `Left/Right` 修改计数
- `Space` 暂停或继续
- `r` 重开
- `q` / `Esc` 返回主界面

贪吃蛇：

- `Arrows` / `WASD` 控制方向
- `i` 在手动和 AI 控制之间切换
- `Space` 暂停或继续
- `r` 重开
- `q` / `Esc` 返回主界面

俄罗斯方块：

- `Left/Right` 或 `A/D` 左右移动
- `Up` / `W` / `X` 顺时针旋转
- `Down` / `S` 软降
- `Enter` 硬降
- `Space` 暂停或继续
- `r` 重开
- `q` / `Esc` 返回主界面

## 开发

运行测试：

```bash
cargo test
```

当前项目还没有单元测试，`cargo test` 主要用于保证能够成功编译。

## 文档

- [ARCHITECTURE.md](./ARCHITECTURE.md)
- [TODO.md](./TODO.md)
- [AGENTS.md](./AGENTS.md)
