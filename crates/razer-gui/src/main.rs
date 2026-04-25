use std::cell::RefCell;
use std::rc::Rc;

mod backend;
mod callbacks;
mod localization;
mod state;
mod window;

use backend::{Backend, connect_device};
use callbacks::wire_callbacks;
use window::{create_tray_icon, start_tray_event_timer, wire_window_events};

slint::include_modules!();

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!("{} starting with Slint UI...", localization::APP_NAME);

    let ui = AppWindow::new()?;
    let backend = Rc::new(RefCell::new(Backend::new()));
    let window_visible = Rc::new(RefCell::new(false));

    wire_callbacks(&ui, backend.clone());
    wire_window_events(&ui, window_visible.clone());
    connect_device(&ui, &backend);

    let tray_state = create_tray_icon(ui.get_locale().as_str())?;
    let _tray_timer = start_tray_event_timer(&ui, window_visible, tray_state.exit_item_id.clone());

    ui.hide()?;
    slint::run_event_loop_until_quit()?;
    Ok(())
}
