use std::process::Command;
use std::{env, fs};
use x_win::get_active_window;

#[cfg(target_os = "windows")]
pub fn get_sys_uuid() -> Option<String> {
    let output = Command::new("wmic").args(["csproduct", "get", "uuid"]).output().ok()?;
    let output_str = String::from_utf8_lossy(&output.stdout);
    output_str.lines().nth(1)?.trim().to_string().into()
}

#[cfg(target_os = "linux")]
pub fn get_sys_uuid() -> Option<String> {
    let path = "/sys/class/dmi/id/product_uuid";
    fs::read_to_string(path).ok().map(|uuid| uuid.trim().to_string())
}

#[cfg(target_os = "macos")]
pub fn get_sys_uuid() -> Option<String> {
    let output = Command::new("ioreg")
        .args(["-rd1", "-c", "IOPlatformExpertDevice"])
        .output()
        .ok()?;
    let output_str = String::from_utf8_lossy(&output.stdout);
    output_str
        .lines()
        .find(|line| line.contains("IOPlatformUUID"))
        .and_then(|line| line.split('=').nth(1))
        .map(|uuid| uuid.trim().trim_matches('"').to_string())
}

pub fn get_focused_window() -> Option<String> {
    if env::var("XDG_SESSION_DESKTOP").unwrap_or_default().contains("Hyprland") {
        let output = Command::new("hyprctl").arg("activewindow").output().unwrap();

        if !output.status.success() {
            return None;
        }

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

#[cfg(any(target_os = "linux", target_os = "macos"))]
pub fn get_hostname() -> Option<String> {
    fs::read_to_string("/etc/hostname").ok().map(|name| name.trim().to_string())
}

#[cfg(target_os = "windows")]
pub fn get_hostname() -> Option<String> {
    let output = Command::new("hostname").output().ok()?;
    String::from_utf8_lossy(&output.stdout).trim().to_string().into()
}
