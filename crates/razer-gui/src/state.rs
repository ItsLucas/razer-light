use razer_core::commands::DeviceState;
use razer_core::types::{BatteryCare, CpuBoost, FanMode, GpuBoost, LogoMode, PerfMode};

use crate::AppWindow;
use crate::localization;

pub fn apply_state(ui: &AppWindow, state: &DeviceState) {
    let locale = ui.get_locale();
    let locale = locale.as_str();

    ui.set_status_text(localization::tr(locale, "status.ready"));
    ui.set_perf_mode(state.perf_mode.label().into());
    ui.set_fan_mode(fan_mode_label(state.fan_mode).into());
    ui.set_cpu_fan_text(
        format!(
            "{}: {} RPM",
            localization::translate(locale, "metric.cpu_fan"),
            state.fan1_rpm
        )
        .into(),
    );
    ui.set_gpu_fan_text(
        format!(
            "{}: {} RPM",
            localization::translate(locale, "metric.gpu_fan"),
            state.fan2_rpm
        )
        .into(),
    );
    ui.set_fan_target_rpm(if state.fan_mode == FanMode::Manual {
        state.fan1_rpm as f32
    } else {
        2500.0
    });

    if let Some(battery_care) = state.battery_care {
        ui.set_battery_limit(battery_care.percent() as f32);
        ui.set_battery_text(
            format!(
                "{}: {}%",
                localization::translate(locale, "metric.battery_limit"),
                battery_care.percent()
            )
            .into(),
        );
    } else {
        ui.set_battery_text(
            format!(
                "{}: {}",
                localization::translate(locale, "metric.battery_limit"),
                localization::translate(locale, "label.unsupported")
            )
            .into(),
        );
    }

    let keyboard_percent = (state.kbd_brightness as f32 / 255.0 * 100.0).round();
    ui.set_keyboard_brightness(keyboard_percent);
    ui.set_keyboard_text(
        format!(
            "{}: {}%",
            localization::translate(locale, "metric.keyboard_brightness"),
            keyboard_percent as i32
        )
        .into(),
    );

    ui.set_cpu_boost(
        state
            .cpu_boost
            .map(|boost| boost.label())
            .unwrap_or("Unavailable")
            .into(),
    );
    ui.set_gpu_boost(
        state
            .gpu_boost
            .map(|boost| boost.label())
            .unwrap_or("Unavailable")
            .into(),
    );
    ui.set_logo_mode(
        state
            .logo_mode
            .map(logo_mode_label)
            .unwrap_or("Unsupported")
            .into(),
    );
}

pub fn set_disconnected_text(ui: &AppWindow) {
    let locale = ui.get_locale();
    let locale = locale.as_str();
    ui.set_device_name(localization::tr(locale, "status.no_device"));
    ui.set_status_text(localization::tr(locale, "status.disconnected"));
    ui.set_cpu_fan_text(
        format!(
            "{}: 0 RPM",
            localization::translate(locale, "metric.cpu_fan")
        )
        .into(),
    );
    ui.set_gpu_fan_text(
        format!(
            "{}: 0 RPM",
            localization::translate(locale, "metric.gpu_fan")
        )
        .into(),
    );
    ui.set_battery_text(
        format!(
            "{}: 80%",
            localization::translate(locale, "metric.battery_limit")
        )
        .into(),
    );
    ui.set_keyboard_text(
        format!(
            "{}: 0%",
            localization::translate(locale, "metric.keyboard_brightness")
        )
        .into(),
    );
}

pub fn parse_perf_mode(value: &str) -> Option<PerfMode> {
    match value {
        "Silent" => Some(PerfMode::Silent),
        "Balanced" => Some(PerfMode::Balanced),
        "Performance" => Some(PerfMode::Performance),
        "Battery Saver" => Some(PerfMode::Battery),
        "Custom" => Some(PerfMode::Custom),
        "Hyperboost" => Some(PerfMode::Hyperboost),
        _ => None,
    }
}

pub fn parse_cpu_boost(value: &str) -> Option<CpuBoost> {
    match value {
        "Low" => Some(CpuBoost::Low),
        "Medium" => Some(CpuBoost::Medium),
        "High" => Some(CpuBoost::High),
        "Boost" => Some(CpuBoost::Boost),
        "Overclock" => Some(CpuBoost::Overclock),
        _ => None,
    }
}

pub fn parse_gpu_boost(value: &str) -> Option<GpuBoost> {
    match value {
        "Low" => Some(GpuBoost::Low),
        "Medium" => Some(GpuBoost::Medium),
        "High" => Some(GpuBoost::High),
        _ => None,
    }
}

pub fn parse_logo_mode(value: &str) -> Option<LogoMode> {
    match value {
        "Off" => Some(LogoMode::Off),
        "Static" => Some(LogoMode::Static),
        "Breathing" => Some(LogoMode::Breathing),
        _ => None,
    }
}

pub fn battery_care_from_percent(percent: i32) -> Option<BatteryCare> {
    let snapped_percent = match percent {
        value if value < 53 => 50,
        value if value < 58 => 55,
        value if value < 63 => 60,
        value if value < 68 => 65,
        value if value < 73 => 70,
        value if value < 78 => 75,
        value if value < 90 => 80,
        _ => 100,
    };

    match snapped_percent {
        50 => Some(BatteryCare::Percent50),
        55 => Some(BatteryCare::Percent55),
        60 => Some(BatteryCare::Percent60),
        65 => Some(BatteryCare::Percent65),
        70 => Some(BatteryCare::Percent70),
        75 => Some(BatteryCare::Percent75),
        80 => Some(BatteryCare::Percent80),
        100 => Some(BatteryCare::Disabled),
        _ => None,
    }
}

pub fn fan_mode_label(mode: FanMode) -> &'static str {
    match mode {
        FanMode::Auto => "Auto",
        FanMode::Manual => "Manual",
    }
}

pub fn logo_mode_label(mode: LogoMode) -> &'static str {
    match mode {
        LogoMode::Off => "Off",
        LogoMode::Static => "Static",
        LogoMode::Breathing => "Breathing",
    }
}
