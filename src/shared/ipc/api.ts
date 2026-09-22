import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { ClipView, Settings, SourceApp } from "../types";

export const api = {
  getClips: () => invoke<ClipView[]>("get_clips"),
  copyClip: (id: string) => invoke<boolean>("copy_clip", { id }),
  pasteClip: (id: string) => invoke<boolean>("paste_clip", { id }),
  pastePlain: (id: string) => invoke<boolean>("paste_plain", { id }),
  togglePin: (id: string) => invoke<boolean | null>("toggle_pin", { id }),
  deleteClip: (id: string) => invoke<boolean>("delete_clip", { id }),
  clearHistory: () => invoke<boolean>("clear_history"),
  getSettings: () => invoke<Settings>("get_settings"),
  updateSettings: (settings: Settings) => invoke<Settings>("update_settings", { settings }),
  getClipThumbnail: (id: string) => invoke<string | null>("get_clip_thumbnail", { id }),
  getAppIcon: (exeName: string, exePath?: string) =>
    invoke<string | null>("get_app_icon", { exeName, exePath }),
  getKnownApps: () => invoke<SourceApp[]>("get_known_apps"),
  hideWindow: () => invoke<void>("hide_window"),
  openSettings: () => invoke<void>("open_settings"),
  onClipsChanged: (callback: () => void): Promise<UnlistenFn> => {
    return listen("clips-changed", () => callback());
  },
};
