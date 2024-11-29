use std::io::Read;

use anyhow::Result;
use genawaiter::{sync::gen, yield_, Generator};
use tokio::{
    time,
    time::{Duration, Interval},
};
use wl_clipboard_rs::paste::{
    get_contents as get_clip_wayland, ClipboardType, MimeType as WaylandMimeType, Seat,
};
use x11_clipboard::Clipboard as X11Clipboard;

use super::{detect_window_manager, WindowManager as WM};
use crate::database::ClipEntry;

#[inline]
fn get_interval() -> Interval {
    time::interval(Duration::from_secs(1))
}

async fn listen_for_clips_x11() -> impl Generator<Yield = ClipEntry, Return = ()> {
    let client = X11Clipboard::new().expect("Failed to initialize X11 clipboard");
    let mut interval = get_interval();
    let timeout = std::time::Duration::from_secs(3);
    let mut previous_content = Vec::<u8>::new();

    gen!({
        let maybe_clip = client.load(
            client.setter.atoms.clipboard,
            client.setter.atoms.utf8_string,
            client.setter.atoms.property,
            timeout,
        );

        if let Ok(contents) = maybe_clip {
            let new_contents = contents;
            if new_contents != previous_content {
                yield_!(ClipEntry::new(previous_content.as_slice()));
                previous_content = new_contents;
            }
        }

        interval.tick().await;
    })
}

async fn listen_for_clips_wayland() -> impl Generator<Yield = ClipEntry, Return = ()> {
    let mut interval = get_interval();
    let mut previous_content = Vec::<u8>::new();

    gen!({
        if let Ok((mut pipe, _)) = get_clip_wayland(
            ClipboardType::Regular,
            Seat::Unspecified,
            WaylandMimeType::Any,
        ) {
            let mut new_content = Vec::<u8>::new();
            if let Ok(_) = pipe.read_exact(&mut new_content) {
                if new_content != previous_content {
                    yield_!(ClipEntry::new(previous_content.as_slice()));
                    previous_content = new_content;
                }
            }
        }

        interval.tick().await;
    })
}

pub async fn listen_for_clips() -> Result<Box<dyn Generator<Yield = ClipEntry, Return = ()>>> {
    match detect_window_manager() {
        Ok(WM::Wayland) => Ok(Box::new(listen_for_clips_wayland().await)),
        Ok(WM::X11) => Ok(Box::new(listen_for_clips_x11().await)),
        Err(e) => Err(e),
    }
}
