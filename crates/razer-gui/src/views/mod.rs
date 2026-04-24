use iced::widget::{button, column, container, row, slider, text, Column};
use iced::Element;

use razer_core::types::*;

use crate::app::{App, Message};

pub fn connect_view() -> Element<'static, Message> {
    column![
        text("No Razer device connected").size(20),
        text("Searching for your Razer Blade laptop...").size(14),
        button("Connect").on_press(Message::Connect).padding(10),
    ]
    .spacing(15)
    .into()
}

pub fn dashboard_view<'a>(app: &'a App) -> Element<'a, Message> {
    let mut content = Column::new().spacing(20);

    content = content.push(section("Performance Mode", perf_section(app)));
    content = content.push(section("Fans", fan_section(app)));
    content = content.push(section("Battery Care", battery_section(app)));
    content = content.push(section("Keyboard Brightness", kbd_section(app)));

    content.into()
}

fn section<'a>(title: &'static str, content: Element<'a, Message>) -> Element<'a, Message> {
    column![text(title).size(18), container(content).padding(10)]
        .spacing(5)
        .into()
}

fn perf_section<'a>(app: &App) -> Element<'a, Message> {
    let modes = [
        PerfMode::Silent,
        PerfMode::Balanced,
        PerfMode::Performance,
        PerfMode::Custom,
    ];

    let current = app.perf_mode;

    let mode_buttons: Vec<Element<'a, Message>> = modes
        .iter()
        .map(|&mode| {
            let is_active = current == Some(mode);
            let btn = button(text(mode.label()).size(13)).padding(8);
            if is_active { btn.into() } else { btn.on_press(Message::SetPerfMode(mode)).into() }
        })
        .collect();

    let mut col = Column::new().spacing(8);
    col = col.push(row(mode_buttons).spacing(8));

    // Fan mode toggle
    if let Some(fan_mode) = app.fan_mode {
        let toggle_to = match fan_mode {
            FanMode::Auto => FanMode::Manual,
            FanMode::Manual => FanMode::Auto,
        };
        col = col.push(
            row![
                text(format!("Fan: {:?}", fan_mode)).size(14),
                button("Toggle").on_press(Message::SetFanMode(toggle_to)).padding(4),
            ]
            .spacing(10),
        );
    }

    // Boost controls (only in Custom mode)
    if app.perf_mode == Some(PerfMode::Custom) {
        let cpu_boosts = [CpuBoost::Low, CpuBoost::Medium, CpuBoost::High, CpuBoost::Boost];
        let cpu_btns: Vec<Element<'a, Message>> = cpu_boosts
            .iter()
            .map(|&b| {
                let active = app.cpu_boost == Some(b);
                let btn = button(text(b.label()).size(11)).padding(4);
                if active { btn.into() } else { btn.on_press(Message::SetCpuBoost(b)).into() }
            })
            .collect();
        col = col.push(row![text("CPU:").size(13)].push(row(cpu_btns).spacing(4)).spacing(8));

        let gpu_boosts = [GpuBoost::Low, GpuBoost::Medium, GpuBoost::High];
        let gpu_btns: Vec<Element<'a, Message>> = gpu_boosts
            .iter()
            .map(|&b| {
                let active = app.gpu_boost == Some(b);
                let btn = button(text(b.label()).size(11)).padding(4);
                if active { btn.into() } else { btn.on_press(Message::SetGpuBoost(b)).into() }
            })
            .collect();
        col = col.push(row![text("GPU:").size(13)].push(row(gpu_btns).spacing(4)).spacing(8));
    }

    col.into()
}

fn fan_section<'a>(app: &App) -> Element<'a, Message> {
    let mut col = Column::new().spacing(8);

    col = col.push(row![
        text(format!("CPU Fan: {} RPM", app.fan1_rpm)).size(14),
        iced::widget::horizontal_space(),
        text(format!("GPU Fan: {} RPM", app.fan2_rpm)).size(14),
    ]);

    // Only show RPM slider in manual mode
    if app.fan_mode == Some(FanMode::Manual) {
        col = col.push(
            row![
                text("Target:").size(14),
                slider(0..=5500u16, app.fan_target_rpm, |v| {
                    Message::SetFanRpm((v / 100) * 100)
                }),
                text(format!("{} RPM", app.fan_target_rpm)).size(14),
                button("Apply").on_press(Message::ApplyFanRpm).padding(4),
            ]
            .spacing(8),
        );
    }

    col.into()
}

fn battery_section<'a>(app: &App) -> Element<'a, Message> {
    let options = [
        BatteryCare::Percent50,
        BatteryCare::Percent60,
        BatteryCare::Percent70,
        BatteryCare::Percent80,
        BatteryCare::Disabled,
    ];

    let buttons: Vec<Element<'a, Message>> = options
        .iter()
        .map(|&level| {
            let is_active = app.battery_care == Some(level);
            let btn = button(text(level.label()).size(12)).padding(6);
            if is_active { btn.into() } else { btn.on_press(Message::SetBatteryCare(level)).into() }
        })
        .collect();

    row(buttons).spacing(6).into()
}

fn kbd_section<'a>(app: &App) -> Element<'a, Message> {
    row![
        text("🔅").size(16),
        slider(0..=255u8, app.kbd_brightness, Message::SetKbdBrightness),
        text("🔆").size(16),
        text(format!("{}%", app.kbd_brightness as u32 * 100 / 255)).size(14),
        button("Apply").on_press(Message::ApplyKbdBrightness).padding(4),
    ]
    .spacing(8)
    .into()
}
