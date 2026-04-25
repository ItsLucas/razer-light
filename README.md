# RHelper

RHelper is a lightweight Rust utility for controlling supported Razer Blade laptop features, inspired by the simple helper-app style of ASUS GHelper.

RHelper 提供一个轻量的 Rust 工具，用于控制部分 Razer Blade 笔记本功能，定位类似华硕 GHelper 的简洁助手应用。

## Features / 功能

- Slint tray GUI for performance, fan, lighting, and battery controls.
- CLI for debugging and direct device control.
- Automatic Razer HID device detection with optional product ID override in the CLI.
- English and Chinese GUI localization.

- Slint 托盘图形界面：性能、风扇、灯光、电池控制。
- 命令行工具：用于调试和直接控制设备。
- 自动检测 Razer HID 设备；CLI 支持指定 USB Product ID。
- 图形界面支持英语和中文。

## Build / 构建

This project is a Cargo workspace with three crates: `razer-core`, `razer-gui`, and `rhelper-cli`.

```bash
cargo build
cargo run -p razer-gui --bin rhelper
cargo run -p rhelper-cli --bin rhelper-cli -- --help
```

Linux builds may require native GUI/HID development libraries such as GLib/GTK, xkbcommon, and hidapi depending on the target environment.

Linux 环境可能需要安装 GLib/GTK、xkbcommon、hidapi 等原生开发库，具体取决于发行版和构建环境。

## GUI / 图形界面

Run the GUI with:

```bash
cargo run -p razer-gui --bin rhelper
```

The app starts in the tray. Left-click the tray icon to show or hide the window. Use the `EN` and `中文` buttons in the header to switch the GUI language.

程序启动后驻留托盘。左键点击托盘图标显示或隐藏窗口。窗口顶部的 `EN` 和 `中文` 按钮可切换界面语言。

## CLI / 命令行

Show available commands:

```bash
cargo run -p rhelper-cli --bin rhelper-cli -- --help
```

Examples:

```bash
cargo run -p rhelper-cli --bin rhelper-cli -- enumerate
cargo run -p rhelper-cli --bin rhelper-cli -- info
cargo run -p rhelper-cli --bin rhelper-cli -- perf get
cargo run -p rhelper-cli --bin rhelper-cli -- fan get
```

## Notes / 注意事项

- Hardware support depends on the Razer Blade model and firmware command compatibility.
- Some controls, such as CPU/GPU boost and logo lighting, may be unavailable on unsupported devices.
- Changing fan, power, lighting, or battery settings can affect thermals, noise, and battery health. Use carefully.

- 硬件支持取决于具体 Razer Blade 型号以及固件命令兼容性。
- CPU/GPU 加速、Logo 灯等功能在不支持的设备上可能不可用。
- 修改风扇、电源、灯光或电池设置可能影响温度、噪音和电池健康，请谨慎使用。
