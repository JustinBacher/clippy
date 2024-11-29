use std::ptr;

use anyhow::{anyhow, Result};
use itertools::Itertools;
use windows::Win32::{
    Foundation::HWND,
    System::{
        DataExchange::{
            CloseClipboard, EmptyClipboard, GetClipboardData, OpenClipboard, SetClipboardData,
        },
        Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE},
    },
    UI::WindowsAndMessaging::CF_TEXT,
};

fn set_clipboard_text(text: &str) -> Result<()> {
    unsafe {
        if !OpenClipboard(HWND(0)).as_bool() {
            Err("Failed to open clipboard".into())
        }

        EmptyClipboard();

        let h_mem = GlobalAlloc(GMEM_MOVEABLE, (text.len() + 1) as u32);
        if h_mem.is_null() {
            return Err("Failed to allocate memory".into());
        }

        let ptr = GlobalLock(h_mem) as *mut u8;
        if ptr.is_null() {
            GlobalUnlock(h_mem);
            return Err("Failed to lock memory".into());
        }
        ptr.copy_from_nonoverlapping(text.as_ptr(), text.len());
        *ptr.add(text.len()) = 0; // Null-terminate the string
        GlobalUnlock(h_mem);

        if SetClipboardData(CF_TEXT, h_mem).is_null() {
            return Err("Failed to set clipboard data".into());
        }

        CloseClipboard();
        Ok(())
    }
}

fn get_clipboard_text() -> Result<String, String> {
    unsafe {
        if OpenClipboard(HWND(0)).as_bool() {
            let handle = GetClipboardData(CF_TEXT);
            if handle.is_null() {
                CloseClipboard();
                return Err("No text found in clipboard".into());
            }

            let ptr = GlobalLock(handle) as *const u8;
            if ptr.is_null() {
                GlobalUnlock(handle);
                CloseClipboard();
                return Err("Failed to lock memory".into());
            }

            let mut len = 0;
            while *ptr.add(len) != 0 {
                len += 1;
            }
            let slice = std::slice::from_raw_parts(ptr, len);
            let text = String::from_utf8_lossy(slice).into_owned();

            GlobalUnlock(handle);
            CloseClipboard();

            Ok(text)
        } else {
            Err("Failed to open clipboard".into())
        }
    }
}

fn get_active_window_title() -> Option<String> {
    unsafe {
        let hwnd: HWND = GetForegroundWindow();
        if hwnd == null_mut() {
            return None;
        }

        let length = GetWindowTextLengthW(hwnd);
        if length == 0 {
            return None;
        }

        let mut buffer: Vec<u16> = vec![0; (length + 1) as usize];
        let read_length = GetWindowTextW(hwnd, &mut buffer);
        if read_length == 0 {
            return None;
        }

        Some(String::from_utf16_lossy(&buffer[..read_length as usize]))
    }
}
