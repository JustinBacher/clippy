use cocoa::appkit::{NSPasteboard, NSPasteboardTypeString};
use cocoa::foundation::{NSData, NSString};

use objc::rc::autoreleasepool;

fn set_clipboard_text(text: &str) {
    autoreleasepool(|| {
        let pasteboard = NSPasteboard::generalPasteboard();
        pasteboard.clearContents();
        let ns_string = NSString::alloc(nil).init_str(text);
        pasteboard.setString_forType(ns_string, NSPasteboardTypeString);
    });
}

fn get_clipboard_text() -> Option<String> {
    autoreleasepool(|| {
        let pasteboard = NSPasteboard::generalPasteboard();
        let ns_string = pasteboard.stringForType(NSPasteboardTypeString);
        if ns_string.is_null() {
            None
        } else {
            Some(unsafe { NSString::stringWithUTF8String(ns_string).to_string() })
        }
    })
}

fn get_active_window_title() -> Option<String> {
    autoreleasepool(|| {
        let active_app = NSWorkspace::sharedWorkspace().frontmostApplication();
        if active_app.is_null() {
            return None;
        }

        let app_name: *const Object = unsafe { msg_send![active_app, localizedName] };
        if app_name.is_null() {
            return None;
        }

        Some(unsafe { NSString::stringWithUTF8String(app_name).to_string() })
    })
}
