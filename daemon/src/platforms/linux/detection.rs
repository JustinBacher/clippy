use std::{env, process::Command, str::FromStr};

use anyhow::{anyhow, Result};
use itertools::Itertools;
use once_cell::sync::Lazy;
use regex::{Regex, RegexBuilder};
use x11rb::connection::Connection as X11Connection;
use x11rb::protocol::xproto::{
    get_property as get_property_x11, AtomEnum as X11AtomEnum, ConnectionExt, Window as X11Window,
};
use zbus::blocking::{Connection as ZbusConnection, Proxy as ZbusProxy};

use super::{Compositor, IntoEnumIterator, WindowManager};

fn get_active_window_sway() -> Result<String> {
    let output = Command::new("swaymsg").arg("-t").arg("get_tree").output()?;

    if !output.status.success() {
        return Err(anyhow!("Failed to query sway IPC"));
    }

    Ok(std::str::from_utf8(&output.stdout)?
        .split("\"focused\":true,")
        .nth(1)
        .and_then(|s| s.split("\"name\":\"").nth(1))
        .and_then(|s| s.split('"').next())
        .unwrap()
        .to_string())
}

fn get_active_window_gnome() -> Result<String> {
    let proxy = ZbusProxy::new(
        &ZbusConnection::session()?,
        "org.gnome.Shell",
        "/org/gnome/Shell",
        "org.gnome.Shell",
    )?;

    Ok(proxy.call("GetWindowTitle", &())?)
}

fn get_active_window_kde() -> Result<String> {
    let proxy = ZbusProxy::new(
        &ZbusConnection::session()?,
        "org.kde.KWin",
        "/KWin",
        "org.kde.KWin",
    )?;

    Ok(proxy.get_property("activeWindowCaption")?)
}

fn get_active_x11_window() -> Result<String> {
    let (ref conn, _) = x11rb::connect(None).expect("Failed to connect to the X server");

    let active_window: X11Window = get_property_x11(
        conn,
        false,
        conn.setup().roots[0].root,
        conn.intern_atom(false, b"_NET_ACTIVE_WINDOW")?.reply()?.atom,
        X11AtomEnum::WINDOW,
        0,
        1024,
    )?
    .reply()?
    .value32()
    .ok_or("Failed to get active window")
    .unwrap()
    .next()
    .unwrap();

    let window_name = get_property_x11(
        conn,
        false,
        active_window,
        conn.intern_atom(false, b"_NET_WM_NAME")?.reply()?.atom,
        conn.intern_atom(false, b"UTF8_STRING")?.reply()?.atom,
        0,
        1024,
    )?;

    Ok(String::from_utf8(window_name.reply()?.value)?)
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
            title += format!(" | {}", line.trim_start_matches("title:").trim()).as_str();
        }
    }

    if title.is_empty() {
        return Err(anyhow!("unable to determine active application"));
    }

    Ok(title)
}

pub fn detect_wayland_compositor() -> Option<Compositor> {
    static COMPOSITOR_RE: Lazy<Regex> = Lazy::new(|| {
        RegexBuilder::new(&Compositor::iter().join("|"))
            .case_insensitive(true)
            .build()
            .unwrap()
    });

    let desktop_session = env::var("XDG_CURRENT_DESKTOP")
        .iter()
        .chain(env::var("XDG_SESSION_DESKTOP").iter())
        .join(" ");

    let found = COMPOSITOR_RE.find(&desktop_session)?.as_str();

    Compositor::from_str(found).ok()
}

pub fn detect_window_manager() -> Result<WindowManager> {
    if let Some(_) = std::env::var_os("WAYLAND_DISPLAY") {
        return Ok(WindowManager::Wayland);
    } else if let Some(_) = std::env::var_os("DISPLAY") {
        return Ok(WindowManager::X11);
    } else {
        Err(anyhow!("Unable to determine "))
    }
}

pub fn get_active_wayland_window() -> Option<String> {
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

pub fn get_active_window_title() -> Option<String> {
    match detect_window_manager() {
        Ok(WindowManager::Wayland) => get_active_wayland_window(),
        Ok(WindowManager::X11) => get_active_x11_window().ok(),
        Err(_) => None,
    }
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
