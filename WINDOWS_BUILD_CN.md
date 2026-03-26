# BlinkSpark Windows 打包与运行说明（中文）

本文档用于在 Windows 环境下将 BlinkSpark 打包为 `exe` 并运行验证。

## 1. 前置环境

### 1.1 安装 Rust（rustup + cargo）

```powershell
winget install -e --id Rustlang.Rustup
```

安装后重开 PowerShell，检查：

```powershell
cargo -V
rustc -V
```

### 1.2 安装 Visual C++ Build Tools（解决 `link.exe`）

如果你使用的是默认 `x86_64-pc-windows-msvc` 目标，需要安装 C++ 编译工具链：

```powershell
winget install -e --id Microsoft.VisualStudio.2022.BuildTools --override "--passive --wait --norestart --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```

安装后重开 PowerShell，检查：

```powershell
where.exe link
where.exe cl
```

## 2. 构建 Release 版本

在项目根目录执行：

```powershell
cd E:\RustroverProjects\BlinkSpark
cargo build --release
```

构建成功后，产物位置：

```text
E:\RustroverProjects\BlinkSpark\target\release\blinkspark.exe
```

## 3. 运行与验证

### 3.1 默认运行

```powershell
.\target\release\blinkspark.exe
```

说明：启动后会出现倒计时小窗（默认右下角，可自由拖动），并按设定间隔循环提醒。
说明：界面默认英文，可在窗口中随时切换中文/英文。
说明：窗口位置会自动保存，下次启动会恢复到上次拖拽位置。
说明：可点击窗口内 `Reset timer`（中文模式为“重置计时”）立即重置倒计时。

### 3.2 快速验证（1 分钟）

```powershell
.\target\release\blinkspark.exe --interval 1
```

### 3.3 循环提醒（默认行为）

```powershell
.\target\release\blinkspark.exe --interval 20
```

### 3.4 仅提醒一次后退出

```powershell
.\target\release\blinkspark.exe --interval 20 --once
```

### 3.5 中文界面/通知

```powershell
.\target\release\blinkspark.exe --lang zh
```

## 4. 常见报错与处理

### 4.1 `error: linker 'link.exe' not found`

原因：未安装或未加载 MSVC 工具链。  
处理：

1. 按第 1.2 节安装 Build Tools。
2. 重开终端后再次执行 `cargo build --release`。
3. 若仍失败，可在同一命令内加载 `vcvars64.bat` 再编译：

```powershell
$vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
$vsPath = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
cmd /c "`"$vsPath\VC\Auxiliary\Build\vcvars64.bat`" && cargo build --release"
```

### 4.2 没有弹出通知

请检查：

1. Windows 通知开关是否开启。
2. 专注助手/勿扰模式是否关闭。
3. 是否等待到了设定时间（建议先用 `--interval 1` 测试）。

## 5. 发布到 GitHub Release（手动）

1. 打开仓库 `Releases` -> `Draft a new release`。
2. 先选择或创建 Tag（例如 `v1.0.0`）。
3. 填写 Release title 与 Release notes。
4. 上传 `target/release/blinkspark.exe`。
5. 点击 `Publish release`。

注意：`Release title` 不是 Tag，发布前必须有有效 Tag。
