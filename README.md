# BlinkSpark

一个跨平台（Linux/macOS/Windows）的 20-20-20 眨眼提醒工具。启动后会显示一个倒计时小窗（默认在右下角，可自由拖动），并按间隔循环提醒。

## 功能
- 支持中英文通知文案
- 支持倒计时桌面小窗（可拖动）
- 自动记住上次窗口位置（下次启动恢复）
- 启动默认英文，可随时切换中文/英文
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

## 通知实现
- Linux: `notify-rust`
- macOS: `mac-notification-sys`
- Windows: `winrt-notification`

如果在某些平台上通知无法显示，通常是系统通知权限/桌面环境设置导致。
