# BlinkSpark

一个跨平台（Linux/macOS/Windows）的 20-20-20 眨眼提醒工具。启动后会显示一个倒计时小窗（默认在右下角，可自由拖动），并按间隔循环提醒。

## 为什么推荐 BlinkSpark
- 真正能长期开着用：轻量、安静、不打断工作流
- 真正跨平台可落地：Linux / macOS / Windows 一套体验
- 真正面向高强度屏幕工作：开发者、设计师、办公人群都适用
- 真正开箱即用：启动即提醒，支持中文/英文、循环/单次模式

如果你经常一坐就是几小时，直到眼睛酸胀才想起来该休息，BlinkSpark 就是那个“存在感刚刚好”的护眼搭子。它不会喧宾夺主，但会稳定地把你的注意力从屏幕里拉回来，帮你把健康习惯变成可持续的日常。

## 功能
- 支持中英文通知文案
- 支持倒计时桌面小窗（可拖动）
- 自动记住上次窗口位置（下次启动恢复）
- 启动默认英文，可随时切换中文/英文
- 悬浮窗支持一键重置倒计时（Reset timer）
- 20-20-20 规则提醒（20 分钟、20 英尺外、20 秒）
- 系统通知 + 倒计时窗口
- 默认循环提醒
- 可选仅提醒一次后退出

## 使用

### 默认（英文 + 持续循环提醒）
```bash
blinkspark
```

### 中文界面/通知
```bash
blinkspark --lang zh
```

### 自定义间隔（分钟）
```bash
blinkspark --interval 30
```

### 仅提醒一次后退出
```bash
blinkspark --once
```

## 开发与构建

需要 Rust 工具链（`cargo`）。

```bash
cargo build --release
```

生成的可执行文件在 `target/release/blinkspark`。

## 平台打包文档

- Windows: `WINDOWS_BUILD_CN.md`
- macOS: `MACOS_BUILD_CN.md`

## 适合人群
- 长时间写代码、看文档、回消息的开发者
- 需要稳定专注又容易忽略休息的知识工作者
- 想要一个“小而美、可控、无负担”提醒工具的人

## 通知实现
- Linux: `notify-rust`
- macOS: `mac-notification-sys`
- Windows: `winrt-notification`

如果在某些平台上通知无法显示，通常是系统通知权限/桌面环境设置导致。
