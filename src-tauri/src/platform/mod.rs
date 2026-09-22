//! Windows platform layer.
//!
//! Nothing above this module talks to Win32 directly. Keeping the OS surface in
//! one place is what lets the rest of the app (and the UI) stay plain data.

pub mod app;
pub mod clipboard_raw;
pub mod icon;
pub mod input;
