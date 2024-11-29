mod clipboard;
mod detection;

pub use clipboard::listen_for_clips;
use derive_more::Display;
use detection::detect_window_manager;
pub use detection::get_active_window_title;
use strum::{EnumIter, EnumString, IntoEnumIterator};

#[derive(EnumIter, EnumString, Debug, PartialEq, Display)]
enum Compositor {
    Gnome,
    Kde,
    Hyprland,
    Sway,
}

#[derive(Debug, PartialEq)]
enum WindowManager {
    X11,
    Wayland,
}
