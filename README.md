# BlinkSpark

一个跨平台（Linux/macOS/Windows）的 20-20-20 眨眼提醒工具。默认行为是等待 20 分钟后提醒一次并退出。

## 功能
- 支持中英文通知文案
- 20-20-20 规则提醒（20 分钟、20 英尺外、20 秒）
- 仅系统通知（不输出多余终端信息）
- 可选循环提醒

## 使用

### 默认（中文 + 提醒一次）
```bash
blinkspark
```

### 英文
```bash
blinkspark --lang en
```

### 自定义间隔（分钟）
```bash
blinkspark --interval 30
```

### 循环提醒（每隔 interval 提醒一次）
```bash
blinkspark --repeat
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
