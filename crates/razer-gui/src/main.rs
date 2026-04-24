mod app;
mod views;

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    log::info!("RazerLight starting...");

    iced::application("RazerLight", app::App::update, app::App::view)
        .theme(app::App::theme)
        .window_size(iced::Size::new(480.0, 640.0))
        .run_with(app::App::new)
        .map_err(|e| anyhow::anyhow!("GUI error: {e}"))?;

    Ok(())
}
