//! Executable icon extraction, used by the clip source indicator.
//!
//! Windows only hands out icons as GDI handles, so the icon is drawn into a
//! 32-bit device-independent bitmap and re-encoded as PNG. Results are cached by
//! the caller (`AppState::app_icon`) because this is comparatively expensive.

use std::path::Path;

use windows_sys::Win32::Foundation::HWND;
use windows_sys::Win32::Graphics::Gdi::{
    CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetDC, ReleaseDC, SelectObject,
    BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HGDIOBJ,
};
use windows_sys::Win32::UI::Shell::SHFILEINFOW;
use windows_sys::Win32::UI::WindowsAndMessaging::{DestroyIcon, DrawIconEx, DI_NORMAL, HICON};

#[link(name = "shell32")]
extern "system" {
    fn SHGetFileInfoW(
        pszpath: *const u16,
        dwfileattributes: u32,
        psfi: *mut SHFILEINFOW,
        cbfileinfo: u32,
        uflags: u32,
    ) -> usize;
}

use super::clipboard_raw::to_wide;

const SHGFI_ICON: u32 = 0x0000_0100;
const SHGFI_LARGEICON: u32 = 0x0000_0000;
const ICON_SIZE: i32 = 32;

/// Returns the application icon as PNG bytes, or `None` when Windows has no
/// icon for the executable. A missing icon is never an error: the UI falls back
/// to a neutral glyph.
pub fn executable_icon_png(executable: &Path) -> Option<Vec<u8>> {
    let hicon = load_icon(executable)?;
    let rgba = unsafe { icon_to_rgba(hicon, ICON_SIZE) };
    unsafe { DestroyIcon(hicon) };

    let rgba = rgba?;
    encode_png(&rgba, ICON_SIZE as u32)
}

fn load_icon(executable: &Path) -> Option<HICON> {
    let wide = to_wide(&executable.to_string_lossy());
    let mut info: SHFILEINFOW = unsafe { std::mem::zeroed() };

    let result = unsafe {
        SHGetFileInfoW(
            wide.as_ptr(),
            0,
            &mut info,
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        )
    };

    if result == 0 || info.hIcon as isize == 0 {
        return None;
    }

    Some(info.hIcon)
}

/// Draws `hicon` into an off-screen bitmap and returns straight RGBA pixels.
unsafe fn icon_to_rgba(hicon: HICON, size: i32) -> Option<Vec<u8>> {
    let screen_dc = GetDC(0 as HWND);
    if screen_dc as isize == 0 {
        return None;
    }

    let mem_dc = CreateCompatibleDC(screen_dc);
    if mem_dc as isize == 0 {
        ReleaseDC(0 as HWND, screen_dc);
        return None;
    }

    let mut info: BITMAPINFO = std::mem::zeroed();
    info.bmiHeader = BITMAPINFOHEADER {
        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
        biWidth: size,
        // Negative height gives a top-down bitmap, matching PNG row order.
        biHeight: -size,
        biPlanes: 1,
        biBitCount: 32,
        biCompression: BI_RGB,
        biSizeImage: 0,
        biXPelsPerMeter: 0,
        biYPelsPerMeter: 0,
        biClrUsed: 0,
        biClrImportant: 0,
    };

    let mut bits: *mut core::ffi::c_void = std::ptr::null_mut();
    let bitmap = CreateDIBSection(
        mem_dc,
        &info,
        DIB_RGB_COLORS,
        &mut bits,
        0 as _,
        0,
    );

    let mut pixels = None;

    if bitmap as isize != 0 && !bits.is_null() {
        let previous: HGDIOBJ = SelectObject(mem_dc, bitmap as HGDIOBJ);
        let drawn = DrawIconEx(mem_dc, 0, 0, hicon, size, size, 0, 0 as _, DI_NORMAL);

        if drawn != 0 {
            let len = (size * size * 4) as usize;
            let bgra = std::slice::from_raw_parts(bits as *const u8, len);
            let mut rgba = Vec::with_capacity(len);

            for chunk in bgra.chunks_exact(4) {
                rgba.extend_from_slice(&[chunk[2], chunk[1], chunk[0], chunk[3]]);
            }

            pixels = Some(rgba);
        }

        SelectObject(mem_dc, previous);
        DeleteObject(bitmap as HGDIOBJ);
    }

    DeleteDC(mem_dc);
    ReleaseDC(0 as HWND, screen_dc);

    pixels
}

fn encode_png(rgba: &[u8], size: u32) -> Option<Vec<u8>> {
    let image = image::RgbaImage::from_raw(size, size, rgba.to_vec())?;
    let mut bytes = Vec::new();

    image
        .write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Png,
        )
        .ok()?;

    Some(bytes)
}
