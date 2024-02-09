use std::io::Cursor;
use winit::window::Icon;

use bevy::{app::Startup, prelude::NonSend, winit::WinitWindows};

fn set_window_icon(winit_windows: NonSend<WinitWindows>) {
    let (icon_rgba, icon_width, icon_height) = {
        let icon_buf = Cursor::new(include_bytes!("../assets/icon.png"));
        let rgba = image::load(icon_buf, image::ImageFormat::Png)
            .expect("Failed to open icon path.")
            .into_rgba8();

        let (width, height) = rgba.dimensions();
        let icon_raw = rgba.into_raw();
        (icon_raw, width, height)
    };

    let icon = Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to load icon.");
    winit_windows.windows.values().for_each(|window| {
        window.set_window_icon(Some(icon.clone()));
    });
}

fn main() {
    let mut app = box_world::app();

    app.add_systems(Startup, set_window_icon);
    app.run();
}
