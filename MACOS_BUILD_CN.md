# BlinkSpark macOS 打包与安装（中文）

本文档适用于在 macOS 上构建 `BlinkSpark.app`，重点覆盖 Intel Mac（含 Monterey 12.x）可运行验证。

## 1. 结论先看

- 默认打包命令会生成 **Universal** 应用（同时包含 `x86_64` + `arm64`）。
- Intel Mac 可直接运行 Universal 包。
- 也支持只打 Intel 包：`MACOS_BUILD_TARGET=x86_64`。
- Windows 不能完整完成该项目的 macOS `.app` 打包（需 Apple 工具链）。

## 2. 前置依赖（在 macOS 执行）

### 2.1 安装 Xcode Command Line Tools（必需）

```bash
xcode-select --install
```

检查：

```bash
clang --version
iconutil --help >/dev/null && echo "iconutil ok"
lipo -h >/dev/null && echo "lipo ok"
sips --version
```

### 2.2 安装 Rust（必需）

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
```

检查：

```bash
cargo -V
rustc -V
rustup -V
```

## 3. 打包命令

在项目根目录执行：

```bash
cd /Users/<你的用户名>/RustroverProjects/BlinkSpark
./scripts/package_macos.sh
```

说明：

- 不传参数时，默认 `MACOS_BUILD_TARGET=universal`。
- 脚本会自动：
1. 生成 macOS 图标
2. 安装/复用 `cargo-bundle`
3. 分别构建 `aarch64-apple-darwin` 与 `x86_64-apple-darwin`
4. 用 `lipo` 合并成 Universal 应用
5. 自动校验二进制架构

### 3.1 可选：只构建 Intel 包

```bash
MACOS_BUILD_TARGET=x86_64 ./scripts/package_macos.sh
```

### 3.2 可选：只构建 Apple Silicon 包

```bash
MACOS_BUILD_TARGET=arm64 ./scripts/package_macos.sh
```

### 3.3 可选：打包后安装到当前用户 Applications

```bash
INSTALL_TO_APPLICATIONS=1 ./scripts/package_macos.sh
```

安装路径：`$HOME/Applications/BlinkSpark.app`

## 4. 产物路径

- Universal（默认）：
  `target/universal-apple-darwin/release/bundle/osx/BlinkSpark.app`
- Intel：
  `target/x86_64-apple-darwin/release/bundle/osx/BlinkSpark.app`
- Apple Silicon：
  `target/aarch64-apple-darwin/release/bundle/osx/BlinkSpark.app`

## 5. Intel Monterey 验证命令（可直接复制）

以下命令用于 Intel Mac（例如 Monterey 12.7.6）验证包是否正确：

```bash
cd /Users/<你的用户名>/RustroverProjects/BlinkSpark
./scripts/package_macos.sh

APP="target/universal-apple-darwin/release/bundle/osx/BlinkSpark.app"
BIN="$APP/Contents/MacOS/BlinkSpark"

# 1) 查看二进制包含的架构，必须同时有 x86_64 和 arm64
lipo -archs "$BIN"

# 2) 查看文件信息（应显示 Mach-O universal）
file "$BIN"

# 3) 如有隔离属性，清理后再打开
xattr -dr com.apple.quarantine "$APP" || true

# 4) 启动应用
open "$APP"
```

如果你只构建了 Intel 包，请把 `APP` 改为：

```bash
APP="target/x86_64-apple-darwin/release/bundle/osx/BlinkSpark.app"
```

## 6. 首次运行常见提示

### 6.1 “来自未识别开发者”

本地未签名构建会触发 Gatekeeper。可用以下任一方式放行：

1. Finder 右键应用 -> 打开
2. 系统设置 -> 隐私与安全性 -> 允许该应用

### 6.2 “这台 Mac 不支持此应用程序”

先检查架构：

```bash
lipo -archs "target/universal-apple-darwin/release/bundle/osx/BlinkSpark.app/Contents/MacOS/BlinkSpark"
```

Intel 机器至少要看到 `x86_64`。若缺失，重新执行默认打包命令生成 Universal 包。

## 7. 脚本参数速查

```bash
./scripts/package_macos.sh --help
```

支持：

- `MACOS_BUILD_TARGET=universal|x86_64|arm64`
- `INSTALL_TO_APPLICATIONS=1`
