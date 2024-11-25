use std::ptr;
use windows::{
    Win32::Foundation::HWND,
    Win32::System::DataExchange::{
        CloseClipboard, EmptyClipboard, GetClipboardData, OpenClipboard, SetClipboardData,
    },
    Win32::System::Memory::{GlobalAlloc, GlobalLock, GlobalUnlock, GMEM_MOVEABLE},
    Win32::UI::WindowsAndMessaging::CF_TEXT,
};

fn set_clipboard_text(text: &str) -> Result<(), String> {
    unsafe {
        // Open the clipboard
        if OpenClipboard(HWND(0)).as_bool() {
            // Clear the clipboard
            EmptyClipboard();

            // Allocate global memory for the text
            let h_mem = GlobalAlloc(GMEM_MOVEABLE, (text.len() + 1) as u32);
            if h_mem.is_null() {
                return Err("Failed to allocate memory".into());
            }

            // Lock the memory and copy the text
            let ptr = GlobalLock(h_mem) as *mut u8;
            if ptr.is_null() {
                GlobalUnlock(h_mem);
                return Err("Failed to lock memory".into());
            }
            ptr.copy_from_nonoverlapping(text.as_ptr(), text.len());
            *ptr.add(text.len()) = 0; // Null-terminate the string
            GlobalUnlock(h_mem);

            // Set the clipboard data
            if SetClipboardData(CF_TEXT, h_mem).is_null() {
                return Err("Failed to set clipboard data".into());
            }

            // Close the clipboard
            CloseClipboard();
            Ok(())
        } else {
            Err("Failed to open clipboard".into())
        }
    }
}

fn get_clipboard_text() -> Result<String, String> {
    unsafe {
        // Open the clipboard
        if OpenClipboard(HWND(0)).as_bool() {
            // Get clipboard data
            let handle = GetClipboardData(CF_TEXT);
            if handle.is_null() {
                CloseClipboard();
                return Err("No text found in clipboard".into());
            }

            // Lock the handle to get a pointer to the text
            let ptr = GlobalLock(handle) as *const u8;
            if ptr.is_null() {
                GlobalUnlock(handle);
                CloseClipboard();
                return Err("Failed to lock memory".into());
            }

            // Read the null-terminated string
            let mut len = 0;
            while *ptr.add(len) != 0 {
                len += 1;
            }
            let slice = std::slice::from_raw_parts(ptr, len);
            let text = String::from_utf8_lossy(slice).into_owned();

            // Unlock and close the clipboard
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
        // Get the handle to the foreground window
        let hwnd: HWND = GetForegroundWindow();
        if hwnd == null_mut() {
            return None;
        }

        // Get the length of the window title
        let length = GetWindowTextLengthW(hwnd);
        if length == 0 {
            return None;
        }

        // Allocate buffer and get the window title
        let mut buffer: Vec<u16> = vec![0; (length + 1) as usize];
        let read_length = GetWindowTextW(hwnd, &mut buffer);
        if read_length == 0 {
            return None;
        }

        // Convert the title to a Rust string
        Some(String::from_utf16_lossy(&buffer[..read_length as usize]))
    }
}
