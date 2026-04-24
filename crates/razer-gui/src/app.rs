#![allow(dead_code)]

use iced::widget::{button, column, container, row, text, Column};
use iced::{Element, Length, Task, Theme};

use std::sync::{Arc, Mutex};

use razer_core::commands::{self, DeviceState};
use razer_core::device::Device;
use razer_core::types::*;

use crate::views;

// ── State ────────────────────────────────────────────────────────────────────

pub struct App {
    pub status: ConnectionStatus,
    pub device_name: String,
    device: Option<Arc<Mutex<Device>>>,

    pub perf_mode: Option<PerfMode>,
    pub fan_mode: Option<FanMode>,
    pub cpu_boost: Option<CpuBoost>,
    pub gpu_boost: Option<GpuBoost>,
    pub battery_care: Option<BatteryCare>,
    pub kbd_brightness: u8,
    pub fan1_rpm: u16,
    pub fan2_rpm: u16,
    pub fan_target_rpm: u16,
    pub logo_mode: Option<LogoMode>,

    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Disconnected,
    Connected,
    Error(String),
}

// ── Messages ─────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Message {
    Connect,
    Connected(Result<ConnectPayload, String>),
    Refresh,
    StateRefreshed(Result<DeviceState, String>),

    SetPerfMode(PerfMode),
    SetCpuBoost(CpuBoost),
    SetGpuBoost(GpuBoost),
    SetFanMode(FanMode),
    SetFanRpm(u16),
    ApplyFanRpm,
    SetBatteryCare(BatteryCare),
    SetKbdBrightness(u8),
    ApplyKbdBrightness,
    SetLogoMode(LogoMode),

    HidResult(Result<(), String>),
    Error(String),
    DismissError,
}

#[derive(Debug, Clone)]
pub struct ConnectPayload {
    pub name: String,
    pub state: DeviceState,
}

// ── App ──────────────────────────────────────────────────────────────────────

impl App {
    pub fn new() -> (Self, Task<Message>) {
        let app = Self {
            status: ConnectionStatus::Disconnected,
            device_name: String::new(),
            device: None,
            perf_mode: None,
            fan_mode: None,
            cpu_boost: None,
            gpu_boost: None,
            battery_care: None,
            kbd_brightness: 0,
            fan1_rpm: 0,
            fan2_rpm: 0,
            fan_target_rpm: 2500,
            logo_mode: None,
            error: None,
        };
        // Auto-connect on start
        (app, Task::perform(async { blocking_connect() }, Message::Connected))
    }

    pub fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn apply_state(&mut self, s: &DeviceState) {
        self.perf_mode = Some(s.perf_mode);
        self.fan_mode = Some(s.fan_mode);
        self.fan1_rpm = s.fan1_rpm;
        self.fan2_rpm = s.fan2_rpm;
        self.battery_care = s.battery_care;
        self.kbd_brightness = s.kbd_brightness;
        self.cpu_boost = s.cpu_boost;
        self.gpu_boost = s.gpu_boost;
        self.logo_mode = s.logo_mode;
    }

    /// Run a blocking closure on the device, then refresh state.
    fn dev_cmd<F>(&self, f: F) -> Task<Message>
    where
        F: FnOnce(&Device) -> anyhow::Result<()> + Send + 'static,
    {
        let Some(dev) = self.device.clone() else {
            return Task::none();
        };
        Task::perform(
            async move {
                let d = dev.lock().map_err(|e| e.to_string())?;
                f(&d).map_err(|e| e.to_string())
            },
            |r| match r {
                Ok(()) => Message::Refresh,
                Err(e) => Message::Error(e),
            },
        )
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Connect => {
                self.error = None;
                Task::perform(async { blocking_connect() }, Message::Connected)
            }

            Message::Connected(result) => {
                match result {
                    Ok(payload) => {
                        // Actually connect now (in main thread, since we can't pass Arc through Message)
                        match Device::auto_open() {
                            Ok(dev) => {
                                self.device = Some(Arc::new(Mutex::new(dev)));
                                self.status = ConnectionStatus::Connected;
                                self.device_name = payload.name;
                                self.apply_state(&payload.state);
                            }
                            Err(e) => {
                                self.status = ConnectionStatus::Error(e.to_string());
                            }
                        }
                    }
                    Err(e) => {
                        self.status = ConnectionStatus::Error(e);
                    }
                }
                Task::none()
            }

            Message::Refresh => {
                let Some(dev) = self.device.clone() else {
                    return Task::none();
                };
                Task::perform(
                    async move {
                        let d = dev.lock().map_err(|e| e.to_string())?;
                        commands::read_all_state(&d).map_err(|e| e.to_string())
                    },
                    Message::StateRefreshed,
                )
            }

            Message::StateRefreshed(Ok(state)) => {
                self.apply_state(&state);
                Task::none()
            }
            Message::StateRefreshed(Err(e)) => {
                self.error = Some(e);
                Task::none()
            }

            Message::SetPerfMode(mode) => {
                self.perf_mode = Some(mode);
                self.dev_cmd(move |d| commands::set_perf_mode(d, mode, FanMode::Auto))
            }

            Message::SetCpuBoost(boost) => {
                self.cpu_boost = Some(boost);
                self.dev_cmd(move |d| commands::set_cpu_boost(d, boost))
            }

            Message::SetGpuBoost(boost) => {
                self.gpu_boost = Some(boost);
                self.dev_cmd(move |d| commands::set_gpu_boost(d, boost))
            }

            Message::SetFanMode(mode) => {
                let perf = self.perf_mode.unwrap_or(PerfMode::Balanced);
                self.fan_mode = Some(mode);
                self.dev_cmd(move |d| commands::set_perf_mode(d, perf, mode))
            }

            Message::SetFanRpm(rpm) => {
                self.fan_target_rpm = (rpm / 100) * 100;
                Task::none()
            }

            Message::ApplyFanRpm => {
                let rpm = self.fan_target_rpm;
                self.dev_cmd(move |d| commands::set_fan_rpm(d, rpm))
            }

            Message::SetBatteryCare(level) => {
                self.battery_care = Some(level);
                self.dev_cmd(move |d| commands::set_battery_care(d, level))
            }

            Message::SetKbdBrightness(b) => {
                self.kbd_brightness = b;
                Task::none()
            }

            Message::ApplyKbdBrightness => {
                let b = self.kbd_brightness;
                self.dev_cmd(move |d| commands::set_keyboard_brightness(d, b))
            }

            Message::SetLogoMode(mode) => {
                self.logo_mode = Some(mode);
                self.dev_cmd(move |d| commands::set_logo_mode(d, mode))
            }

            Message::HidResult(Err(e)) | Message::Error(e) => {
                self.error = Some(e);
                Task::none()
            }
            Message::HidResult(Ok(())) => Task::none(),

            Message::DismissError => {
                self.error = None;
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let content: Element<Message> = match &self.status {
            ConnectionStatus::Disconnected => views::connect_view(),
            ConnectionStatus::Connected => views::dashboard_view(self),
            ConnectionStatus::Error(e) => {
                column![
                    text("Connection Error").size(24),
                    text(e.clone()).size(14),
                    button("Retry").on_press(Message::Connect),
                ]
                .spacing(10)
                .into()
            }
        };

        let mut main_col = Column::new().spacing(10).padding(20).width(Length::Fill);

        main_col = main_col.push(
            row![
                text("⚡ RazerLight").size(28),
                iced::widget::horizontal_space(),
                text(&self.device_name).size(14),
                button("🔄").on_press(Message::Refresh).padding(4),
            ]
            .spacing(10),
        );

        if let Some(err) = &self.error {
            main_col = main_col.push(
                container(
                    row![
                        text(format!("⚠ {err}")).size(13),
                        button("✕").on_press(Message::DismissError),
                    ]
                    .spacing(10),
                )
                .padding(8),
            );
        }

        main_col = main_col.push(content);

        container(main_col)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

// ── Blocking helpers (run on tokio::spawn_blocking via Task::perform) ────────

fn blocking_connect() -> Result<ConnectPayload, String> {
    let device = Device::auto_open().map_err(|e| e.to_string())?;
    let name = device.name().to_string();
    let state = commands::read_all_state(&device).map_err(|e| e.to_string())?;
    Ok(ConnectPayload { name, state })
}
