# TODO

## 当前状态

- [x] 使用 `ratatui` 跑通最小程序入口
- [x] 建立 `main.rs + app.rs + game/` 的最小结构
- [x] 用 `App` 保存状态
- [x] 建立最小 `Game` trait
- [x] 把 `Counter Demo` 抽到 `game/` 目录
- [x] 用 `crossterm` 读取键盘输入
- [x] 实现计数器加减和退出
- [x] 用 `Block` 和 `Paragraph` 画出最小界面
- [x] 支持多个界面
- [x] 增加选择游戏界面
- [x] 支持从选择页进入 `Counter Demo`
- [x] 支持从游戏页返回选择页
- [x] 由 `App` 控制暂停状态

## 下一步

- [ ] 把不同界面的绘制拆成更小的函数
- [ ] 把选择游戏界面做成真正的列表
- [ ] 给 `GameKind` 增加显示名称等元数据
- [ ] 增加第二个游戏验证 `Game` trait
- [ ] 继续收敛 app 层和 game 层职责边界

## 暂时不做

- [ ] 多游戏框架
- [ ] `runtime.rs`
- [ ] 贪吃蛇
- [ ] 俄罗斯方块
