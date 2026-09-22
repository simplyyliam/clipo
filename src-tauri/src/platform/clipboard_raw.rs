//! Raw Win32 clipboard access for the formats `arboard` does not cover:
//! file drops (`CF_HDROP`), rich text (`HTML Format`), the change counter and
//! the flags applications use to opt out of clipboard history.
//!
//! Everything here is a thin, single-purpose wrapper. Higher level policy lives
//! in `crate::clipboard`.

use std::mem::size_of;
use std::path::{Path, PathBuf};
use std::time::Duration;

use windows_sys::Win32::Foundation::{HANDLE, HWND, POINT};
use windows_sys::Win32::System::DataExchange::{
    CloseClipboard, EmptyClipboard, GetClipboardData, GetClipboardSequenceNumber,
    IsClipboardFormatAvailable, OpenClipboard, RegisterClipboardFormatW, SetClipboardData,
};
use windows_sys::Win32::System::Memory::{
    GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE,
};
use windows_sys::Win32::UI::Shell::{DragQueryFileW, DROPFILES, HDROP};

pub const CF_HDROP: u32 = 15;

/// Number that Windows increments on every clipboard change. Polling this is
/// essentially free, so the monitor only reads clipboard contents when it moves.
pub fn sequence_number() -> u32 {
    unsafe { GetClipboardSequenceNumber() }
}

/// RAII guard around the clipboard, which is a global single-owner resource.
/// Opening can fail while another process holds it, so it is retried briefly.
pub struct ClipboardGuard;

impl ClipboardGuard {
    pub fn open() -> Option<Self> {
        for attempt in 0..10u64 {
            let opened = unsafe { OpenClipboard(0 as HWND) };
            if opened != 0 {
                return Some(Self);
            }
            std::thread::sleep(Duration::from_millis(5 * (attempt + 1)));
        }

        log::warn!("could not open the Windows clipboard");
        None
    }
}

impl Drop for ClipboardGuard {
    fn drop(&mut self) {
        unsafe { CloseClipboard() };
    }
}

pub fn register_format(name: &str) -> u32 {
    let wide = to_wide(name);
    unsafe { RegisterClipboardFormatW(wide.as_ptr()) }
}

/// Requires an open clipboard.
pub fn has_format(format: u32) -> bool {
    unsafe { IsClipboardFormatAvailable(format) != 0 }
}

/// Copies the bytes of a clipboard format out of the global memory block.
/// Requires an open clipboard.
pub fn format_bytes(format: u32) -> Option<Vec<u8>> {
    unsafe {
        let handle = GetClipboardData(format);
        if handle as isize == 0 {
            return None;
        }

        let ptr = GlobalLock(handle as _);
        if ptr.is_null() {
            return None;
        }

        let size = GlobalSize(handle as _);
        let bytes = std::slice::from_raw_parts(ptr as *const u8, size).to_vec();
        GlobalUnlock(handle as _);

        Some(bytes)
    }
}

/// Paths of a file drop on the clipboard. Requires an open clipboard.
pub fn read_file_drop() -> Vec<PathBuf> {
    unsafe {
        let handle = GetClipboardData(CF_HDROP);
        if handle as isize == 0 {
            return Vec::new();
        }

        let drop_handle = handle as HDROP;
        let count = DragQueryFileW(drop_handle, u32::MAX, std::ptr::null_mut(), 0);
        let mut paths = Vec::with_capacity(count as usize);

        for index in 0..count {
            let len = DragQueryFileW(drop_handle, index, std::ptr::null_mut(), 0);
            if len == 0 {
                continue;
            }

            let mut buffer = vec![0u16; len as usize + 1];
            let written = DragQueryFileW(
                drop_handle,
                index,
                buffer.as_mut_ptr(),
                buffer.len() as u32,
            );
            if written == 0 {
                continue;
            }

            buffer.truncate(written as usize);
            paths.push(PathBuf::from(String::from_utf16_lossy(&buffer)));
        }

        paths
    }
}

/// Replaces the clipboard with a file drop, so copied file clips behave like a
/// copy made in Explorer.
pub fn write_file_drop(paths: &[impl AsRef<Path>]) -> bool {
    if paths.is_empty() {
        return false;
    }

    let mut wide: Vec<u16> = Vec::new();
    for path in paths {
        wide.extend(to_wide(&path.as_ref().to_string_lossy()));
    }
    wide.push(0);

    let header = size_of::<DROPFILES>();
    let total = header + wide.len() * size_of::<u16>();

    unsafe {
        let Some(_guard) = ClipboardGuard::open() else {
            return false;
        };

        if EmptyClipboard() == 0 {
            return false;
        }

        let block = GlobalAlloc(GMEM_MOVEABLE, total);
        if block as isize == 0 {
            return false;
        }

        let base = GlobalLock(block as _) as *mut u8;
        if base.is_null() {
            return false;
        }

        let drop_files = DROPFILES {
            pFiles: header as u32,
            pt: POINT { x: 0, y: 0 },
            fNC: 0,
            fWide: 1,
        };
        std::ptr::write(base as *mut DROPFILES, drop_files);
        std::ptr::copy_nonoverlapping(
            wide.as_ptr() as *const u8,
            base.add(header),
            wide.len() * size_of::<u16>(),
        );
        GlobalUnlock(block as _);

        // On success Windows owns the memory block.
        SetClipboardData(CF_HDROP, block as HANDLE) as isize != 0
    }
}

pub fn to_wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}
