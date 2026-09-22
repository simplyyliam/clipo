import { useState, useEffect, useCallback } from "react";
import type { Settings } from "../types";
import { api } from "../ipc/api";

const DEFAULT_SETTINGS: Settings = {
  shortcut: "Ctrl+Shift+V",
  historyLimit: 200,
  excludedApps: [],
  blockSensitive: true,
  theme: "system",
  launchAtStartup: false,
};

export function useSettings() {
  const [settings, setSettings] = useState<Settings>(DEFAULT_SETTINGS);
  const [loading, setLoading] = useState(true);

  const loadSettings = useCallback(async () => {
    try {
      const data = await api.getSettings();
      setSettings(data);
      applyTheme(data.theme);
    } catch (err) {
      console.error("Failed to load settings:", err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadSettings();
  }, [loadSettings]);

  const updateSettings = async (newSettings: Settings) => {
    try {
      const updated = await api.updateSettings(newSettings);
      setSettings(updated);
      applyTheme(updated.theme);
      return updated;
    } catch (err) {
      console.error("Failed to update settings:", err);
      throw err;
    }
  };

  return {
    settings,
    loading,
    updateSettings,
    refresh: loadSettings,
  };
}

function applyTheme(theme: Settings["theme"]) {
  const root = document.documentElement;
  if (theme === "dark") {
    root.classList.add("dark");
  } else if (theme === "light") {
    root.classList.remove("dark");
  } else {
    const isDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
    if (isDark) {
      root.classList.add("dark");
    } else {
      root.classList.remove("dark");
    }
  }
}
