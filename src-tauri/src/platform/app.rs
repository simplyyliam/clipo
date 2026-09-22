//! App-awareness: which application owns the foreground window, and how to
//! give focus back to it.
//!
//! This is the single implementation of "which app is the user in". Clip source
//! indicators, excluded applications and Quick Paste all consume it.

use std::path::PathBuf;

use windows_sys::Win32::Foundation::{CloseHandle, HWND, MAX_PATH, POINT};
use windows_sys::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetCursorPos, GetForegroundWindow, GetWindowThreadProcessId, SetForegroundWindow,
};

/// Window handle kept as a raw pointer-sized value so it can live in shared
/// state across threads.
pub type RawWindow = isize;

pub fn foreground_window() -> Option<RawWindow> {
    let handle = unsafe { GetForegroundWindow() } as isize;
    (handle != 0).then_some(handle)
}

/// Brings a previously recorded window back to the foreground. Used by Quick
/// Paste before synthesising the paste keystroke.
pub fn focus_window(window: RawWindow) -> bool {
    if window == 0 {
        return false;
    }
    unsafe { SetForegroundWindow(window as HWND) != 0 }
}

pub fn cursor_position() -> Option<(i32, i32)> {
    let mut point = POINT { x: 0, y: 0 };
    let ok = unsafe { GetCursorPos(&mut point) };
    (ok != 0).then_some((point.x, point.y))
}

/// Full path of the executable that owns `window`.
pub fn window_executable(window: RawWindow) -> Option<PathBuf> {
    if window == 0 {
        return None;
    }

    unsafe {
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(window as HWND, &mut pid);
        if pid == 0 {
            return None;
        }

        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if process as isize == 0 {
            return None;
        }

        let mut buffer = vec![0u16; MAX_PATH as usize];
        let mut len = buffer.len() as u32;
        let ok = QueryFullProcessImageNameW(process, 0, buffer.as_mut_ptr(), &mut len);
        CloseHandle(process);

        if ok == 0 || len == 0 {
            return None;
        }

        buffer.truncate(len as usize);
        Some(PathBuf::from(String::from_utf16_lossy(&buffer)))
    }
}

/// `code.exe` -> `Code`. Windows executables rarely carry a friendly name we can
/// read cheaply, so the file stem is used and only lightly prettified.
pub fn display_name(executable: &PathBuf) -> String {
    let stem = executable
        .file_stem()
        .map(|stem| stem.to_string_lossy().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let cleaned = stem.replace(['_', '-'], " ");
    let mut name = String::with_capacity(cleaned.len());

    for word in cleaned.split_whitespace() {
        if !name.is_empty() {
            name.push(' ');
        }
        let mut chars = word.chars();
        if let Some(first) = chars.next() {
            name.extend(first.to_uppercase());
            name.push_str(chars.as_str());
        }
    }

    if name.is_empty() {
        "Unknown".to_string()
    } else {
        name
    }
}

pub fn executable_name(executable: &PathBuf) -> String {
    executable
        .file_name()
        .map(|name| name.to_string_lossy().to_lowercase())
        .unwrap_or_default()
}
