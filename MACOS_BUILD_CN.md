# BlinkSpark macOS 打包与安装说明（中文）

本文档用于在 macOS 上将 BlinkSpark 打包为 `.app` 并完成安装。

## 1. 前置依赖

### 1.1 Xcode Command Line Tools（必需）

```bash
xcode-select --install
```

检查：

```bash
clang --version
sips --version
iconutil --help >/dev/null && echo "iconutil ok"
```

### 1.2 Rust 工具链（必需）

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

### 1.3 cargo-bundle（用于生成 `.app`）

```bash
cargo install cargo-bundle --locked
```

## 2. 一键打包（推荐）

项目已提供自动脚本，会自动做以下事情：

1. 生成 macOS 图标 `assets/branding/generated/blinkspark.icns`
2. 安装（或复用）`cargo-bundle`
3. 按当前机器架构打包 `.app`

执行：

```bash
cd /Users/shuyixiao/RustroverProjects/BlinkSpark
./scripts/package_macos.sh
```

打包产物路径（按架构区分）：

- Apple Silicon: `target/aarch64-apple-darwin/release/bundle/osx/BlinkSpark.app`
- Intel: `target/x86_64-apple-darwin/release/bundle/osx/BlinkSpark.app`

## 3. 自动安装到应用目录

### 3.1 安装到当前用户应用目录（无需 sudo）

```bash
INSTALL_TO_APPLICATIONS=1 ./scripts/package_macos.sh
```

安装位置：`$HOME/Applications/BlinkSpark.app`

### 3.2 安装到系统应用目录（可选）

```bash
sudo rm -rf /Applications/BlinkSpark.app
sudo cp -R target/*/release/bundle/osx/BlinkSpark.app /Applications/
```

## 4. 手动打包（不使用脚本）

```bash
cd /Users/shuyixiao/RustroverProjects/BlinkSpark
./scripts/generate_macos_icon.sh
rustup target add "$(uname -m | sed 's/arm64/aarch64-apple-darwin/;s/x86_64/x86_64-apple-darwin/')"
cargo bundle --release --format osx
```

## 5. 首次运行与权限

### 5.1 首次打开提示“来自未识别开发者”

因为是本地未签名构建，macOS 可能拦截。可通过以下任一方式放行：

1. Finder 中右键应用 -> `打开`。
2. 系统设置 -> `隐私与安全性` -> 允许该应用。

### 5.2 通知权限

BlinkSpark 依赖系统通知，请在：

- 系统设置 -> `通知` -> `BlinkSpark`

确保允许通知显示。

## 6. 常见问题

### 6.1 `cargo: command not found`

说明 Rust 未安装或环境变量未加载。

```bash
source "$HOME/.cargo/env"
cargo -V
```

### 6.2 `clang` / `iconutil` 不可用

说明未安装 Command Line Tools。

```bash
xcode-select --install
```

### 6.3 打包成功但双击无法打开

执行去隔离属性（尤其是从下载目录复制出的构建产物）：

```bash
xattr -dr com.apple.quarantine /path/to/BlinkSpark.app
```

## 7. 可选：从 `.app` 生成 DMG

`cargo-bundle v0.9.0` 不支持 `--format dmg`。如果需要 `.dmg`，先按本文生成 `.app`，再用 `hdiutil` 打包：

```bash
APP_PATH="target/$(uname -m | sed 's/arm64/aarch64-apple-darwin/;s/x86_64/x86_64-apple-darwin/')/release/bundle/osx/BlinkSpark.app"
mkdir -p dist
cp -R "$APP_PATH" dist/
hdiutil create -volname "BlinkSpark" -srcfolder dist -ov -format UDZO dist/BlinkSpark.dmg
```
