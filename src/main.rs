#![windows_subsystem = "windows"]

mod app;
mod audio;
mod i18n;
mod state;
mod task;
mod ui;

use iced::window;

fn main() -> iced::Result {
    #[cfg(not(target_arch = "wasm32"))]
    tracing_subscriber::fmt::init();

    // Initialize i18n by accessing the lazy static
    std::sync::LazyLock::force(&i18n::LANGUAGE_LOADER);
    audio::init_audio();

    iced::application(app::Todos::new, app::Todos::update, app::Todos::view)
        .subscription(app::Todos::subscription)
        .title(app::Todos::title)
        .font(app::Todos::ICON_FONT)
        .window(window::Settings {
            size: (500.0, 800.0).into(),
            min_size: Some((500.0, 600.0).into()),
            ..window::Settings::default()
        })
        .run()
}

