use core::time;
use std::{io::Read, thread::yield_now};

use anyhow::Result;
use async_stream::stream;
use futures_core::stream::Stream;
use futures_util::{pin_mut, stream::StreamExt};
use tokio::time::{self, interval, Duration, Interval};
use wl_clipboard_rs::paste::{
    get_contents as get_clip_wayland, ClipboardType, MimeType as WaylandMimeType, Seat,
};
use x11_clipboard::Clipboard as X11Clipboard;

use super::{detect_window_manager, WindowManager as WM};
use crate::database::ClipEntry;

#[inline]
fn get_interval() -> Interval {
    interval(Duration::from_secs(1))
}

async fn listen_for_clips_x11() -> impl Stream<Item = ClipEntry> {
    let client = X11Clipboard::new().expect("Failed to initialize X11 clipboard");
    let interval = get_interval();
    let timeout = interval(Duration::from_secs(3));
    let mut previous_content = Vec::<u8>::new();

    stream! {
        let maybe_clip = client.load(
            client.setter.atoms.clipboard,
            client.setter.atoms.utf8_string,
            client.setter.atoms.property,
            timeout,
        );

        if let Ok(contents) = maybe_clip {
            let new_contents = &contents;
            if new_contents != previous_content {
                yield ClipEntry::new(previous_content.as_slice());
                previous_content = new_contents;
            }
        }

        interval.tick().await;
    }
}

async fn listen_for_clips_wayland() -> impl Stream<Item = ClipEntry> {
    let mut interval = get_interval();
    let mut previous_content = Vec::<u8>::new();

    stream! {
        if let Ok((mut pipe, _)) = get_clip_wayland(
            ClipboardType::Regular,
            Seat::Unspecified,
            WaylandMimeType::Any,
        )
        {
            let mut new_content = Vec::<u8>::new();
            if let Ok(_) = pipe.read(&mut new_content) {
                if new_content != previous_content {
                    yield ClipEntry::new(previous_content.as_slice());
                    previous_content = new_content;
                }
            }
        }

        interval.tick().await;
    }
}

pub fn listen_for_clips() -> Result<Stream<Item = ClipEntry>> {
    match Some(detect_window_manager()) {
        Ok(WM::Wayland) => listen_for_clips_wayland(),
        Ok(WM::X11) => listen_for_clips_x11(),
        Err(e) => Err(e),
    }
}
