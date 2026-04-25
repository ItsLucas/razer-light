use razer_core::commands;
use razer_core::types::{FanMode, PerfMode};
use slint::ComponentHandle;

use crate::AppWindow;
use crate::backend::{
    SharedBackend, connect_device, refresh_language, refresh_state, run_device_command,
};
use crate::localization;
use crate::state::{
    battery_care_from_percent, parse_cpu_boost, parse_gpu_boost, parse_logo_mode, parse_perf_mode,
};

pub fn wire_callbacks(ui: &AppWindow, backend: SharedBackend) {
    wire_translation(ui);
    wire_backend_callbacks(ui, backend);
}

fn wire_translation(ui: &AppWindow) {
    ui.on_translate(|locale, key| localization::tr(locale.as_str(), key.as_str()));
}

fn wire_backend_callbacks(ui: &AppWindow, backend: SharedBackend) {
    let weak_ui = ui.as_weak();
    let refresh_backend = backend.clone();
    ui.on_refresh(move || {
        if let Some(ui) = weak_ui.upgrade() {
            refresh_state(&ui, &refresh_backend);
        }
    });

    let weak_ui = ui.as_weak();
    let connect_backend = backend.clone();
    ui.on_connect(move || {
        if let Some(ui) = weak_ui.upgrade() {
            connect_device(&ui, &connect_backend);
        }
    });

    let weak_ui = ui.as_weak();
    let locale_backend = backend.clone();
    ui.on_set_locale(move |locale| {
        if let Some(ui) = weak_ui.upgrade() {
            ui.set_locale(locale);
            refresh_language(&ui, &locale_backend);
        }
    });

    let weak_ui = ui.as_weak();
    let perf_backend = backend.clone();
    ui.on_set_perf_mode(move |mode| {
        if let Some(ui) = weak_ui.upgrade() {
            if let Some(perf_mode) = parse_perf_mode(mode.as_str()) {
                run_device_command(&ui, &perf_backend, |device| {
                    commands::set_perf_mode(device, perf_mode, FanMode::Auto)
                });
            }
        }
    });

    let weak_ui = ui.as_weak();
    let fan_backend = backend.clone();
    ui.on_set_fan_mode(move |mode| {
        if let Some(ui) = weak_ui.upgrade() {
            let fan_mode = if mode.as_str() == "Manual" {
                FanMode::Manual
            } else {
                FanMode::Auto
            };
            let perf_mode =
                parse_perf_mode(ui.get_perf_mode().as_str()).unwrap_or(PerfMode::Balanced);
            run_device_command(&ui, &fan_backend, |device| {
                commands::set_perf_mode(device, perf_mode, fan_mode)
            });
        }
    });

    let weak_ui = ui.as_weak();
    let fan_rpm_backend = backend.clone();
    ui.on_apply_fan_rpm(move |rpm| {
        if let Some(ui) = weak_ui.upgrade() {
            let rounded_rpm = ((rpm as u16) / 100) * 100;
            ui.set_fan_target_rpm(rounded_rpm as f32);
            run_device_command(&ui, &fan_rpm_backend, |device| {
                commands::set_fan_rpm(device, rounded_rpm)
            });
        }
    });

    let weak_ui = ui.as_weak();
    let max_fan_backend = backend.clone();
    ui.on_set_max_fan(move |enabled| {
        if let Some(ui) = weak_ui.upgrade() {
            run_device_command(&ui, &max_fan_backend, |device| {
                commands::set_max_fan_speed(device, enabled)
            });
        }
    });

    let weak_ui = ui.as_weak();
    let cpu_backend = backend.clone();
    ui.on_set_cpu_boost(move |boost| {
        if let Some(ui) = weak_ui.upgrade() {
            if let Some(cpu_boost) = parse_cpu_boost(boost.as_str()) {
                run_device_command(&ui, &cpu_backend, |device| {
                    commands::set_cpu_boost(device, cpu_boost)
                });
            }
        }
    });

    let weak_ui = ui.as_weak();
    let gpu_backend = backend.clone();
    ui.on_set_gpu_boost(move |boost| {
        if let Some(ui) = weak_ui.upgrade() {
            if let Some(gpu_boost) = parse_gpu_boost(boost.as_str()) {
                run_device_command(&ui, &gpu_backend, |device| {
                    commands::set_gpu_boost(device, gpu_boost)
                });
            }
        }
    });

    let weak_ui = ui.as_weak();
    let battery_backend = backend.clone();
    ui.on_set_battery_limit(move |limit| {
        if let Some(ui) = weak_ui.upgrade() {
            if let Some(level) = battery_care_from_percent(limit as i32) {
                run_device_command(&ui, &battery_backend, |device| {
                    commands::set_battery_care(device, level)
                });
            }
        }
    });

    let weak_ui = ui.as_weak();
    let keyboard_backend = backend.clone();
    ui.on_set_keyboard_brightness(move |percent| {
        if let Some(ui) = weak_ui.upgrade() {
            let brightness = ((percent / 100.0) * 255.0).round().clamp(0.0, 255.0) as u8;
            run_device_command(&ui, &keyboard_backend, |device| {
                commands::set_keyboard_brightness(device, brightness)
            });
        }
    });

    let weak_ui = ui.as_weak();
    let logo_backend = backend.clone();
    ui.on_set_logo_mode(move |mode| {
        if let Some(ui) = weak_ui.upgrade() {
            if let Some(logo_mode) = parse_logo_mode(mode.as_str()) {
                run_device_command(&ui, &logo_backend, |device| {
                    commands::set_logo_mode(device, logo_mode)
                });
            }
        }
    });
}
