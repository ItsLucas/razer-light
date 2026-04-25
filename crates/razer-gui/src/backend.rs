use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use razer_core::commands;
use razer_core::device::Device;

use crate::AppWindow;
use crate::localization;
use crate::state::{apply_state, set_disconnected_text};

pub struct Backend {
    device: Option<Arc<Mutex<Device>>>,
}

impl Backend {
    pub fn new() -> Self {
        Self { device: None }
    }
}

pub type SharedBackend = Rc<RefCell<Backend>>;

pub fn connect_device(ui: &AppWindow, backend: &SharedBackend) {
    let locale = ui.get_locale();
    ui.set_status_text(localization::tr(locale.as_str(), "status.connecting"));
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
                Err(error) => ui.set_error_text(
                    format!(
                        "{}: {error}",
                        localization::translate(locale.as_str(), "error.state_read_failed")
                    )
                    .into(),
                ),
            }
        }
        Err(error) => {
            backend.borrow_mut().device = None;
            ui.set_connected(false);
            set_disconnected_text(ui);
            ui.set_error_text(error.to_string().into());
        }
    }
}

pub fn refresh_state(ui: &AppWindow, backend: &SharedBackend) {
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

pub fn refresh_language(ui: &AppWindow, backend: &SharedBackend) {
    if ui.get_connected() {
        refresh_state(ui, backend);
    } else {
        set_disconnected_text(ui);
    }
}

pub fn run_device_command<F>(ui: &AppWindow, backend: &SharedBackend, command: F)
where
    F: FnOnce(&Device) -> anyhow::Result<()>,
{
    let Some(device) = backend.borrow().device.clone() else {
        let locale = ui.get_locale();
        ui.set_error_text(localization::tr(
            locale.as_str(),
            "error.no_device_connected",
        ));
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
