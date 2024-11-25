use futures_core::stream::Stream;

use crate::database::ClipEntry;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

pub fn get_active_window() -> Option<String> {
    #[cfg(target_os = "linux")]
    {
        linux::get_active_window_title()
    }

    #[cfg(target_os = "windows")]
    {
        windows::get_active_window_title()
    }

    #[cfg(target_os = "macos")]
    {
        macos::get_active_window_title()
    }
}

pub fn listen_for_clips() -> Result<Stream<Item = ClipEntry>> {
    #[cfg(target_os = "linux")]
    {
        linux::listen_for_clips()
    }

    #[cfg(target_os = "windows")]
    {
        todo!()
    }

    #[cfg(target_os = "macos")]
    {
        todo!()
    }
}
