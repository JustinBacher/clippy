use std::{env, process::Command, str::FromStr};

use anyhow::{anyhow, Result};
use itertools::Itertools;
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};
use strum::EnumString;
use zbus::blocking::{Connection, Proxy};

const COMPOSITOR_NAMES: &str = "GNOME|KDE|HYPRLAND|SWAY";

#[derive(EnumString, Debug, PartialEq)]
enum Compositor {
    Gnome,
    Kde,
    Hyprland,
    Sway,
}

fn get_active_window_sway() -> Result<String> {
    let output = Command::new("swaymsg").arg("-t").arg("get_tree").output()?;

    if !output.status.success() {
        return Err(anyhow!("Failed to query sway IPC"));
    }

    let title = std::str::from_utf8(&output.stdout)?
        .split("\"focused\":true,")
        .nth(1)
        .and_then(|s| s.split("\"name\":\"").nth(1))
        .and_then(|s| s.split('"').next())
        .unwrap()
        .to_string();

    Ok(title)
}

fn get_active_window_gnome() -> Result<String> {
    let proxy = Proxy::new(
        &Connection::session()?,
        "org.gnome.Shell",
        "/org/gnome/Shell",
        "org.gnome.Shell",
    )?;

    Ok(proxy.call("GetWindowTitle", &())?)
}

fn get_active_window_kde() -> Result<String> {
    let proxy = Proxy::new(
        &Connection::session()?,
        "org.kde.KWin",
        "/KWin",
        "org.kde.KWin",
    )?;

    Ok(proxy.get_property("activeWindowCaption")?)
}

fn get_active_window_hyprland() -> Result<String> {
    let output = Command::new("hyprctl").arg("activewindow").output()?;

    if !output.status.success() {
        return Err(anyhow!("hyprctl failed: {:?}", output.status));
    }

    let mut title = "".to_string();

    for line in String::from_utf8(output.stdout)?.lines() {
        if line.starts_with("initialTitle:") {
            title += line.trim_start_matches("title:").trim();
        } else if line.starts_with("title:") {
            title += format!(" {}", line.trim_start_matches("title:").trim()).as_str();
        }
    }

    if title.is_empty() {
        return Err(anyhow!("unable to determine active application"));
    }

    Ok(title)
}

fn detect_wayland_compositor() -> Option<Compositor> {}

fn detect_wayland_compositor() -> Option<Compositor> {
    static COMPOSITOR_RE: Lazy<Regex> =
        Lazy::new(|| RegexBuilder::new(COMPOSITOR_NAMES).case_insensitive(true).build().unwrap());

    let desktop_session = env::var("XDG_CURRENT_DESKTOP")
        .iter()
        .chain(env::var("XDG_SESSION_DESKTOP").iter())
        .join(" ");

    let found = COMPOSITOR_RE.find(&desktop_session)?.as_str();

    Compositor::from_str(found).ok()
}

pub fn get_active_window() -> Option<String> {
    let compositor = match detect_wayland_compositor() {
        Some(Compositor::Gnome) => get_active_window_gnome(),
        Some(Compositor::Kde) => get_active_window_kde(),
        Some(Compositor::Hyprland) => get_active_window_hyprland(),
        Some(Compositor::Sway) => get_active_window_sway(),
        None => Err(anyhow!("Unable to determine compositor")),
    };
    // TODO: need to log this instead of ignoring it.
    compositor.ok()
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn it_detects_compositor() {
        env::set_var("XDG_CURRENT_DESKTOP", "Gnome");
        assert_eq!(detect_wayland_compositor().unwrap(), Compositor::Gnome);

        env::remove_var("XDG_CURRENT_DESKTOP");
        env::set_var("XDG_SESSION_DESKTOP", "Kde");
        assert_eq!(detect_wayland_compositor().unwrap(), Compositor::Kde);

        env::remove_var("XDG_SESSION_DESKTOP");
        env::set_var("XDG_CURRENT_DESKTOP", "no compositor");
        assert!(detect_wayland_compositor().is_none())
    }
}
