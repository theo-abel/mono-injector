#![windows_subsystem = "windows"]
// Some view message variants and helper items are defined ahead of full UI
// wiring and will be connected as the feature set grows.
#![allow(dead_code)]
#![allow(clippy::struct_excessive_bools, clippy::too_many_lines)]

mod app;
mod theme;
mod util;
mod view;
mod widget;

use app::App;

const WINDOW_SIZE: (f32, f32) = (1280.0, 800.0);
const WINDOW_MIN_SIZE: (f32, f32) = (900.0, 600.0);

fn main() -> iced::Result {
    iced::application("mono-injector", App::update, App::view)
        .window(window_settings())
        .theme(App::theme)
        .subscription(|app: &App| app.subscription())
        .font(include_bytes!("../assets/fonts/HankenGrotesk-Regular.ttf").as_slice())
        .font(include_bytes!("../assets/fonts/HankenGrotesk-SemiBold.ttf").as_slice())
        .font(include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf").as_slice())
        .font(include_bytes!("../assets/fonts/JetBrainsMono-Medium.ttf").as_slice())
        .font(include_bytes!("../assets/fonts/JetBrainsMono-Bold.ttf").as_slice())
        .font(include_bytes!("../assets/fonts/MaterialSymbolsOutlined.ttf").as_slice())
        .run()
}

fn window_settings() -> iced::window::Settings {
    iced::window::Settings {
        size: iced::Size::new(WINDOW_SIZE.0, WINDOW_SIZE.1),
        min_size: Some(iced::Size::new(WINDOW_MIN_SIZE.0, WINDOW_MIN_SIZE.1)),
        ..Default::default()
    }
}
