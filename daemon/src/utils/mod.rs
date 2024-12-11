use std::{env, process::Command};

pub mod async_helpers;
pub mod clipboard;
pub mod config;
pub mod ip;
#[allow(clippy::module_inception)]
pub mod utils;
use x_win::get_active_window;

pub use utils::*;

pub fn get_focused_window() -> Option<String> {
    if env::var("XDG_SESSION_DESKTOP").unwrap_or_default().contains("Hyprland") {
        let output = Command::new("hyprctl").arg("activewindow").output().unwrap();

        if !output.status.success() {
            return None;
        }

        // Parse the output for the window title
        let output_str = String::from_utf8(output.stdout).unwrap();
        for mut line in output_str.lines() {
            line = line.trim();
            if line.trim().starts_with("title:") {
                return Some(line.trim_start_matches("title:").trim().to_string());
            }
        }
        None
    } else if let Ok(wininfo) = get_active_window() {
        Some(wininfo.title)
    } else {
        None
    }
}
