export type ClipKind = "text" | "link" | "image" | "files";

export interface SourceApp {
  name: string;
  exeName: string;
  exePath?: string;
}

export interface ImageMeta {
  width: number;
  height: number;
  byteLen: number;
}

export interface FileRef {
  name: string;
  path: string;
}

export interface ClipView {
  id: string;
  kind: ClipKind;
  createdAt: number;
  pinned: boolean;
  preview: string;
  searchText: string;
  charCount: number;
  url?: string;
  hasHtml: boolean;
  image?: ImageMeta;
  files: FileRef[];
  source?: SourceApp;
}

export type ThemeMode = "system" | "light" | "dark";

export interface Settings {
  shortcut: string;
  historyLimit: number;
  excludedApps: string[];
  blockSensitive: boolean;
  theme: ThemeMode;
  launchAtStartup: boolean;
}

export type FilterCategory = "all" | "pinned" | "text" | "image" | "link" | "files";
