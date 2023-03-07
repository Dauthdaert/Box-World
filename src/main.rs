use std::io::Cursor;

use bevy::{
    prelude::{Entity, NonSend, Query, With},
    window::Window,
    winit::WinitWindows,
};

fn set_window_icon(winit_windows: NonSend<WinitWindows>, windows: Query<Entity, With<Window>>) {
    let window = windows.single();
    let primary = winit_windows
        .get_window(window)
        .expect("Primary window should exist.");

    let (icon_rgba, icon_width, icon_height) = {
        let icon_buf = Cursor::new(include_bytes!("../assets/bevy.png"));
        let rgba = image::load(icon_buf, image::ImageFormat::Png)
            .expect("Failed to open icon path.")
            .into_rgba8();

        let (width, height) = rgba.dimensions();
        let icon_raw = rgba.into_raw();
        (icon_raw, width, height)
    };

    let icon = winit::window::Icon::from_rgba(icon_rgba, icon_width, icon_height)
        .expect("Failed to load icon.");
    primary.set_window_icon(Some(icon));
}

fn main() {
    let mut app = box_world::app();

    app.add_startup_system(set_window_icon);
    app.run();
}
