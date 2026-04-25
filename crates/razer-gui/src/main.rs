use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use razer_core::commands::{self, DeviceState};
use razer_core::device::Device;
use razer_core::types::{BatteryCare, CpuBoost, FanMode, GpuBoost, LogoMode, PerfMode};
use slint::winit_030::{EventResult, WinitWindowAccessor, winit::event::WindowEvent};
use slint::{ComponentHandle, Timer, TimerMode};
use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem};
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

slint::include_modules!();

struct Backend {
    device: Option<Arc<Mutex<Device>>>,
}

struct TrayState {
    _icon: TrayIcon,
    exit_item_id: MenuId,
}

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!("RazerLight starting with Slint UI...");

    let ui = AppWindow::new()?;
    let backend = Rc::new(RefCell::new(Backend { device: None }));
    let window_visible = Rc::new(RefCell::new(false));

    wire_callbacks(&ui, backend.clone());
    wire_window_events(&ui, window_visible.clone());
    connect_device(&ui, &backend);

    let tray_state = create_tray_icon()?;
    let _tray_timer = start_tray_event_timer(&ui, window_visible, tray_state.exit_item_id.clone());

    ui.hide()?;
    slint::run_event_loop_until_quit()?;
    Ok(())
}

fn create_tray_icon() -> anyhow::Result<TrayState> {
    let icon = create_icon()?;

    let menu = Menu::new();
    let exit_item = MenuItem::new("Exit", true, None);
    let exit_item_id = exit_item.id().clone();
    menu.append_items(&[&exit_item])
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;

    let tray_icon = TrayIconBuilder::new()
        .with_icon(icon)
        .with_menu(Box::new(menu))
        .with_menu_on_left_click(false)
        .with_tooltip("RazerLight")
        .build()
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;

    Ok(TrayState {
        _icon: tray_icon,
        exit_item_id,
    })
}

fn create_icon() -> anyhow::Result<Icon> {
    let width = 32;
    let height = 32;
    let mut rgba = Vec::with_capacity(width * height * 4);

    for y in 0..height {
        for x in 0..width {
            let border = x < 3 || y < 3 || x >= width - 3 || y >= height - 3;
            let accent = x > 9 && x < 23 && y > 9 && y < 23;
            let (red, green, blue) = if border {
                (18, 184, 148)
            } else if accent {
                (240, 240, 240)
            } else {
                (30, 30, 30)
            };
            rgba.extend_from_slice(&[red, green, blue, 255]);
        }
    }

    Icon::from_rgba(rgba, width as u32, height as u32)
        .map_err(|error| anyhow::anyhow!(error.to_string()))
}

fn start_tray_event_timer(
    ui: &AppWindow,
    window_visible: Rc<RefCell<bool>>,
    exit_item_id: MenuId,
) -> Timer {
    let timer = Timer::default();
    let weak_ui = ui.as_weak();

    timer.start(TimerMode::Repeated, Duration::from_millis(80), move || {
        while let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == exit_item_id {
                if let Some(ui) = weak_ui.upgrade() {
                    let _ = ui.hide();
                }
                let _ = slint::quit_event_loop();
                return;
            }
        }

        while let Ok(event) = TrayIconEvent::receiver().try_recv() {
            if let TrayIconEvent::Click {
                button,
                button_state,
                ..
            } = event
            {
                if button == MouseButton::Left && button_state == MouseButtonState::Up {
                    let Some(ui) = weak_ui.upgrade() else {
                        continue;
                    };

                    if *window_visible.borrow() {
                        let _ = ui.hide();
                        *window_visible.borrow_mut() = false;
                    } else {
                        let _ = ui.show();
                        *window_visible.borrow_mut() = true;
                        ui.window()
                            .with_winit_window(|window| window.focus_window());
                    }
                }
            }
        }
    });

    timer
}

fn wire_window_events(ui: &AppWindow, window_visible: Rc<RefCell<bool>>) {
    let weak_ui = ui.as_weak();
    ui.window()
        .on_winit_window_event(move |_window, event| match event {
            WindowEvent::Focused(false) => {
                if let Some(ui) = weak_ui.upgrade() {
                    let _ = ui.hide();
                    *window_visible.borrow_mut() = false;
                }
                EventResult::Propagate
            }
            WindowEvent::CloseRequested => {
                if let Some(ui) = weak_ui.upgrade() {
                    let _ = ui.hide();
                    *window_visible.borrow_mut() = false;
                }
                EventResult::PreventDefault
            }
            _ => EventResult::Propagate,
        });
}

fn wire_callbacks(ui: &AppWindow, backend: Rc<RefCell<Backend>>) {
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

fn connect_device(ui: &AppWindow, backend: &Rc<RefCell<Backend>>) {
    ui.set_status_text("Connecting to Razer device...".into());
    ui.set_error_text("".into());

    match Device::auto_open() {
        Ok(device) => {
            let device_name = device.name().to_string();
            let state_result = commands::read_all_state(&device);
            backend.borrow_mut().device = Some(Arc::new(Mutex::new(device)));
            ui.set_device_name(device_name.into());
            ui.set_connected(true);

            match state_result {
                Ok(state) => apply_state(ui, &state),
                Err(error) => ui.set_error_text(format!("State read failed: {error}").into()),
            }
        }
        Err(error) => {
            backend.borrow_mut().device = None;
            ui.set_connected(false);
            ui.set_device_name("No device".into());
            ui.set_status_text("Disconnected".into());
            ui.set_error_text(error.to_string().into());
        }
    }
}

fn refresh_state(ui: &AppWindow, backend: &Rc<RefCell<Backend>>) {
    let Some(device) = backend.borrow().device.clone() else {
        connect_device(ui, backend);
        return;
    };

    let state_result = with_device(&device, commands::read_all_state);
    match state_result {
        Ok(state) => {
            ui.set_error_text("".into());
            apply_state(ui, &state);
        }
        Err(error) => ui.set_error_text(error.to_string().into()),
    }
}

fn run_device_command<F>(ui: &AppWindow, backend: &Rc<RefCell<Backend>>, command: F)
where
    F: FnOnce(&Device) -> anyhow::Result<()>,
{
    let Some(device) = backend.borrow().device.clone() else {
        ui.set_error_text("No Razer device is connected.".into());
        return;
    };

    match with_device(&device, command) {
        Ok(()) => refresh_state(ui, backend),
        Err(error) => ui.set_error_text(error.to_string().into()),
    }
}

fn with_device<T, F>(device: &Arc<Mutex<Device>>, command: F) -> anyhow::Result<T>
where
    F: FnOnce(&Device) -> anyhow::Result<T>,
{
    let guard = device
        .lock()
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;
    command(&guard)
}

fn apply_state(ui: &AppWindow, state: &DeviceState) {
    ui.set_status_text("Ready".into());
    ui.set_perf_mode(state.perf_mode.label().into());
    ui.set_fan_mode(fan_mode_label(state.fan_mode).into());
    ui.set_cpu_fan_text(format!("CPU Fan: {} RPM", state.fan1_rpm).into());
    ui.set_gpu_fan_text(format!("GPU Fan: {} RPM", state.fan2_rpm).into());
    ui.set_fan_target_rpm(if state.fan_mode == FanMode::Manual {
        state.fan1_rpm as f32
    } else {
        2500.0
    });

    if let Some(battery_care) = state.battery_care {
        ui.set_battery_limit(battery_care.percent() as f32);
        ui.set_battery_text(format!("Battery Charge Limit: {}%", battery_care.percent()).into());
    } else {
        ui.set_battery_text("Battery Charge Limit: Unsupported".into());
    }

    let keyboard_percent = (state.kbd_brightness as f32 / 255.0 * 100.0).round();
    ui.set_keyboard_brightness(keyboard_percent);
    ui.set_keyboard_text(format!("Keyboard Brightness: {}%", keyboard_percent as i32).into());

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

fn parse_perf_mode(value: &str) -> Option<PerfMode> {
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

fn parse_cpu_boost(value: &str) -> Option<CpuBoost> {
    match value {
        "Low" => Some(CpuBoost::Low),
        "Medium" => Some(CpuBoost::Medium),
        "High" => Some(CpuBoost::High),
        "Boost" => Some(CpuBoost::Boost),
        "Overclock" => Some(CpuBoost::Overclock),
        _ => None,
    }
}

fn parse_gpu_boost(value: &str) -> Option<GpuBoost> {
    match value {
        "Low" => Some(GpuBoost::Low),
        "Medium" => Some(GpuBoost::Medium),
        "High" => Some(GpuBoost::High),
        _ => None,
    }
}

fn parse_logo_mode(value: &str) -> Option<LogoMode> {
    match value {
        "Off" => Some(LogoMode::Off),
        "Static" => Some(LogoMode::Static),
        "Breathing" => Some(LogoMode::Breathing),
        _ => None,
    }
}

fn battery_care_from_percent(percent: i32) -> Option<BatteryCare> {
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

fn fan_mode_label(mode: FanMode) -> &'static str {
    match mode {
        FanMode::Auto => "Auto",
        FanMode::Manual => "Manual",
    }
}

fn logo_mode_label(mode: LogoMode) -> &'static str {
    match mode {
        LogoMode::Off => "Off",
        LogoMode::Static => "Static",
        LogoMode::Breathing => "Breathing",
    }
}
