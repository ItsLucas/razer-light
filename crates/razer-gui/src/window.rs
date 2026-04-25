use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use slint::winit_030::{EventResult, WinitWindowAccessor, winit::event::WindowEvent};
use slint::{ComponentHandle, Timer, TimerMode};
use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem};
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

use crate::AppWindow;
use crate::localization;

pub struct TrayState {
    _icon: TrayIcon,
    pub exit_item_id: MenuId,
}

pub fn create_tray_icon(locale: &str) -> anyhow::Result<TrayState> {
    let icon = create_icon()?;

    let menu = Menu::new();
    let exit_item = MenuItem::new(localization::translate(locale, "action.exit"), true, None);
    let exit_item_id = exit_item.id().clone();
    menu.append_items(&[&exit_item])
        .map_err(|error| anyhow::anyhow!(error.to_string()))?;

    let tray_icon = TrayIconBuilder::new()
        .with_icon(icon)
        .with_menu(Box::new(menu))
        .with_menu_on_left_click(false)
        .with_tooltip(localization::APP_NAME)
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

pub fn start_tray_event_timer(
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

pub fn wire_window_events(ui: &AppWindow, window_visible: Rc<RefCell<bool>>) {
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
