# BlinkSpark

一个跨平台（Linux/macOS/Windows）的 20-20-20 眨眼提醒工具。默认行为是持续运行，并按间隔循环提醒。

## 功能
- 支持中英文通知文案
- 20-20-20 规则提醒（20 分钟、20 英尺外、20 秒）
- 仅系统通知（不输出多余终端信息）
- 默认循环提醒
- 可选仅提醒一次后退出

## 使用

### 默认（中文 + 持续循环提醒）
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
