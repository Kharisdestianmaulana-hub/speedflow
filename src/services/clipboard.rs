use url::Url;

pub struct ClipboardService;

impl ClipboardService {
    pub fn is_valid_download_url(text: &str) -> bool {
        let trimmed = text.trim();
        if trimmed.contains(char::is_whitespace) {
            return false;
        }
        if let Ok(parsed) = Url::parse(trimmed) {
            (parsed.scheme() == "http" || parsed.scheme() == "https") && parsed.has_host()
        } else {
            false
        }
    }

    pub fn get_clipboard_url() -> Option<String> {
        let text = Self::read_clipboard_text()?;
        let trimmed = text.trim();
        if Self::is_valid_download_url(trimmed) {
            Some(trimmed.to_string())
        } else {
            None
        }
    }

    #[cfg(windows)]
    pub fn set_clipboard_text(text: &str) -> bool {
        const CF_UNICODETEXT: u32 = 13;
        const GMEM_MOVEABLE: u32 = 0x0002;
        unsafe {
            unsafe extern "system" {
                fn OpenClipboard(hWndNewOwner: isize) -> i32;
                fn CloseClipboard() -> i32;
                fn EmptyClipboard() -> i32;
                fn SetClipboardData(uFormat: u32, hMem: isize) -> isize;
                fn GlobalAlloc(uFlags: u32, dwBytes: usize) -> isize;
                fn GlobalLock(hMem: isize) -> *mut u16;
                fn GlobalUnlock(hMem: isize) -> i32;
            }

            if OpenClipboard(0) == 0 {
                return false;
            }
            EmptyClipboard();

            let utf16: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
            let byte_len = utf16.len() * std::mem::size_of::<u16>();
            let hmem = GlobalAlloc(GMEM_MOVEABLE, byte_len);
            if hmem == 0 {
                CloseClipboard();
                return false;
            }

            let ptr = GlobalLock(hmem);
            if ptr.is_null() {
                CloseClipboard();
                return false;
            }

            std::ptr::copy_nonoverlapping(utf16.as_ptr(), ptr, utf16.len());
            GlobalUnlock(hmem);

            let ok = SetClipboardData(CF_UNICODETEXT, hmem) != 0;
            CloseClipboard();
            ok
        }
    }

    #[cfg(not(windows))]
    pub fn set_clipboard_text(_text: &str) -> bool {
        false
    }

    #[cfg(windows)]
    fn read_clipboard_text() -> Option<String> {
        const CF_UNICODETEXT: u32 = 13;
        unsafe {
            unsafe extern "system" {
                fn OpenClipboard(hWndNewOwner: isize) -> i32;
                fn CloseClipboard() -> i32;
                fn GetClipboardData(uFormat: u32) -> isize;
                fn GlobalLock(hMem: isize) -> *mut u16;
                fn GlobalUnlock(hMem: isize) -> i32;
            }

            if OpenClipboard(0) == 0 {
                return None;
            }

            let hmem = GetClipboardData(CF_UNICODETEXT);
            if hmem == 0 {
                CloseClipboard();
                return None;
            }

            let ptr = GlobalLock(hmem);
            if ptr.is_null() {
                CloseClipboard();
                return None;
            }

            let mut len = 0usize;
            while len < 8192 && *ptr.add(len) != 0 {
                len += 1;
            }
            let slice = std::slice::from_raw_parts(ptr, len);
            let result = String::from_utf16(slice).ok();

            GlobalUnlock(hmem);
            CloseClipboard();
            result
        }
    }

    #[cfg(not(windows))]
    fn read_clipboard_text() -> Option<String> {
        None
    }
}
