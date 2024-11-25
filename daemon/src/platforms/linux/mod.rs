mod clipboard;
mod detection;

use derive_more::Display;
use strum::{EnumIter, EnumString, IntoEnumIterator};

pub use clipboard::listen_for_clips;
pub use detection::get_active_window_title;
use detection::{detect_wayland_compositor, detect_window_manager};

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
