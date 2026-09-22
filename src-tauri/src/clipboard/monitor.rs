//! Clipboard Monitor: polls the Win32 clipboard sequence number in a background
//! thread and invokes the ingestion pipeline whenever it changes.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use tauri::AppHandle;

use crate::clipboard;
use crate::platform::clipboard_raw;
use crate::state::AppState;

pub struct MonitorHandle {
    running: Arc<AtomicBool>,
}

impl MonitorHandle {
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

/// Spawns the clipboard monitoring background thread.
pub fn start(app: AppHandle, state: Arc<AppState>) -> MonitorHandle {
    let running = Arc::new(AtomicBool::new(true));
    let is_running = Arc::clone(&running);

    std::thread::Builder::new()
        .name("clipo-clipboard-monitor".into())
        .spawn(move || {
            let mut last_seq = clipboard_raw::sequence_number();

            while is_running.load(Ordering::Relaxed) {
                let current_seq = clipboard_raw::sequence_number();

                if current_seq != last_seq {
                    last_seq = current_seq;
                    let result = clipboard::ingest(&app, &state);
                    log::debug!("Clipboard ingested: {result:?}");
                }

                std::thread::sleep(Duration::from_millis(250));
            }
        })
        .expect("failed to spawn clipboard monitor thread");

    MonitorHandle { running }
}
