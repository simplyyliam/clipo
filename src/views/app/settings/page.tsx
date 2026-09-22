import { useState, useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { useSettings } from "@/shared/hooks/useSettings";
import { api } from "@/shared/ipc/api";
import { Button } from "@/components/ui/button";
import {
  ArrowLeft,
  Shield,
  Palette,
  Keyboard,
  History,
  Power,
  Trash2,
  Check,
  Ban,
  Plus,
  X,
} from "lucide-react";
import type { ThemeMode } from "@/shared/types";

export default function SettingsPage() {
  const navigate = useNavigate();
  const { settings, updateSettings, loading } = useSettings();
  const [shortcut, setShortcut] = useState(settings.shortcut);
  const [historyLimit, setHistoryLimit] = useState(settings.historyLimit);
  const [blockSensitive, setBlockSensitive] = useState(settings.blockSensitive);
  const [theme, setTheme] = useState<ThemeMode>(settings.theme);
  const [launchAtStartup, setLaunchAtStartup] = useState(settings.launchAtStartup);
  const [excludedApps, setExcludedApps] = useState<string[]>(settings.excludedApps);
  const [newExcludedApp, setNewExcludedApp] = useState("");
  const [knownApps, setKnownApps] = useState<{ exeName: string; name: string }[]>([]);
  const [savedSuccess, setSavedSuccess] = useState(false);

  useEffect(() => {
    setShortcut(settings.shortcut);
    setHistoryLimit(settings.historyLimit);
    setBlockSensitive(settings.blockSensitive);
    setTheme(settings.theme);
    setLaunchAtStartup(settings.launchAtStartup);
    setExcludedApps(settings.excludedApps);
  }, [settings]);

  useEffect(() => {
    api.getKnownApps().then((apps) => {
      setKnownApps(apps);
    });
  }, []);

  const handleSave = async () => {
    await updateSettings({
      shortcut,
      historyLimit,
      blockSensitive,
      theme,
      launchAtStartup,
      excludedApps,
    });
    setSavedSuccess(true);
    setTimeout(() => setSavedSuccess(false), 2000);
  };

  const handleAddExcluded = (appExe: string) => {
    const clean = appExe.trim().toLowerCase();
    if (!clean || excludedApps.includes(clean)) return;
    setExcludedApps([...excludedApps, clean]);
    setNewExcludedApp("");
  };

  const handleRemoveExcluded = (appExe: string) => {
    setExcludedApps(excludedApps.filter((a) => a !== appExe));
  };

  const handleClearHistory = async () => {
    if (confirm("Are you sure you want to clear all unpinned clipboard history?")) {
      await api.clearHistory();
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full p-8 text-muted-foreground text-sm">
        Loading settings...
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full bg-background text-foreground overflow-y-auto no-scrollbar select-none p-4 gap-6">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-border/40 pb-3">
        <div className="flex items-center gap-2">
          <Button
            variant="ghost"
            size="icon"
            className="h-7 w-7 text-muted-foreground hover:text-foreground -ml-1"
            onClick={() => navigate("/")}
          >
            <ArrowLeft className="w-4 h-4" />
          </Button>
          <h1 className="text-base font-semibold tracking-tight">Preferences</h1>
        </div>

        <Button
          size="sm"
          onClick={handleSave}
          className="h-7 px-3 text-xs bg-primary text-primary-foreground font-medium rounded-md shadow-xs gap-1.5"
        >
          {savedSuccess ? (
            <>
              <Check className="w-3.5 h-3.5 text-emerald-300" />
              <span>Saved</span>
            </>
          ) : (
            "Save Changes"
          )}
        </Button>
      </div>

      <div className="flex flex-col gap-6 text-xs">
        {/* Appearance Section */}
        <div className="flex flex-col gap-3">
          <div className="flex items-center gap-2 font-medium text-foreground/90">
            <Palette className="w-4 h-4 text-primary" />
            <span>Appearance & Theme</span>
          </div>
          <div className="grid grid-cols-3 gap-2">
            {(["system", "light", "dark"] as ThemeMode[]).map((mode) => (
              <button
                key={mode}
                type="button"
                onClick={() => setTheme(mode)}
                className={`py-2 px-3 rounded-lg border text-center capitalize font-medium transition-all ${
                  theme === mode
                    ? "border-primary bg-primary/10 text-primary"
                    : "border-border/60 hover:bg-muted/40 text-muted-foreground"
                }`}
              >
                {mode}
              </button>
            ))}
          </div>
        </div>

        {/* Shortcut Section */}
        <div className="flex flex-col gap-3">
          <div className="flex items-center gap-2 font-medium text-foreground/90">
            <Keyboard className="w-4 h-4 text-primary" />
            <span>Global Activation Shortcut</span>
          </div>
          <div className="flex items-center gap-2">
            <input
              type="text"
              value={shortcut}
              onChange={(e) => setShortcut(e.target.value)}
              placeholder="e.g. Ctrl+Shift+V"
              className="flex-1 bg-muted/40 border border-border/60 rounded-lg px-3 py-1.5 text-xs focus:ring-1 focus:ring-primary focus:outline-hidden font-mono"
            />
          </div>
          <p className="text-[11px] text-muted-foreground">
            Pressing this key combo toggles the Clipo popover on Windows.
          </p>
        </div>

        {/* History Capacity */}
        <div className="flex flex-col gap-3">
          <div className="flex items-center gap-2 font-medium text-foreground/90">
            <History className="w-4 h-4 text-primary" />
            <span>History Limit</span>
          </div>
          <div className="flex items-center gap-3">
            <input
              type="range"
              min="50"
              max="500"
              step="25"
              value={historyLimit}
              onChange={(e) => setHistoryLimit(Number(e.target.value))}
              className="flex-1 accent-primary cursor-pointer"
            />
            <span className="font-mono text-muted-foreground w-12 text-right">
              {historyLimit}
            </span>
          </div>
          <p className="text-[11px] text-muted-foreground">
            Maximum number of unpinned clipboard entries kept in storage.
          </p>
        </div>

        {/* Privacy & Security */}
        <div className="flex flex-col gap-3">
          <div className="flex items-center gap-2 font-medium text-foreground/90">
            <Shield className="w-4 h-4 text-primary" />
            <span>Privacy & Security</span>
          </div>

          <label className="flex items-center justify-between p-2.5 rounded-lg border border-border/40 bg-card/40 cursor-pointer hover:bg-card/70 transition-colors">
            <div className="flex flex-col gap-0.5 pr-2">
              <span className="font-medium text-foreground/90">Block Sensitive Content</span>
              <span className="text-[11px] text-muted-foreground leading-tight">
                Automatically detect and ignore passwords, API keys, JWTs, and credit cards.
              </span>
            </div>
            <input
              type="checkbox"
              checked={blockSensitive}
              onChange={(e) => setBlockSensitive(e.target.checked)}
              className="h-4 w-4 accent-primary rounded cursor-pointer"
            />
          </label>
        </div>

        {/* Excluded Applications */}
        <div className="flex flex-col gap-3">
          <div className="flex items-center gap-2 font-medium text-foreground/90">
            <Ban className="w-4 h-4 text-primary" />
            <span>Excluded Applications</span>
          </div>
          <p className="text-[11px] text-muted-foreground">
            Clips copied while active in these applications will never be captured.
          </p>

          <div className="flex items-center gap-2">
            <input
              type="text"
              value={newExcludedApp}
              onChange={(e) => setNewExcludedApp(e.target.value)}
              placeholder="e.g. 1password.exe, keepass.exe"
              className="flex-1 bg-muted/40 border border-border/60 rounded-lg px-3 py-1.5 text-xs focus:ring-1 focus:ring-primary focus:outline-hidden font-mono"
              onKeyDown={(e) => {
                if (e.key === "Enter") handleAddExcluded(newExcludedApp);
              }}
            />
            <Button
              type="button"
              size="sm"
              variant="outline"
              className="h-7 px-2.5 text-xs gap-1 border-border/60"
              onClick={() => handleAddExcluded(newExcludedApp)}
            >
              <Plus className="w-3.5 h-3.5" />
              <span>Add</span>
            </Button>
          </div>

          {/* Quick suggestions from known apps */}
          {knownApps.length > 0 && (
            <div className="flex items-center gap-1.5 flex-wrap">
              <span className="text-[10px] text-muted-foreground">Recent apps:</span>
              {knownApps.slice(0, 5).map((app) => (
                <button
                  key={app.exeName}
                  type="button"
                  onClick={() => handleAddExcluded(app.exeName)}
                  className="px-2 py-0.5 text-[10px] font-mono bg-muted/60 text-muted-foreground hover:text-foreground rounded border border-border/40 hover:border-border transition-colors"
                >
                  +{app.exeName}
                </button>
              ))}
            </div>
          )}

          {excludedApps.length > 0 && (
            <div className="flex flex-wrap gap-1.5 pt-1">
              {excludedApps.map((app) => (
                <div
                  key={app}
                  className="flex items-center gap-1 px-2 py-1 bg-muted/80 text-foreground font-mono rounded-md border border-border/50 text-[11px]"
                >
                  <span>{app}</span>
                  <button
                    type="button"
                    onClick={() => handleRemoveExcluded(app)}
                    className="text-muted-foreground hover:text-destructive transition-colors ml-0.5"
                  >
                    <X className="w-3 h-3" />
                  </button>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* System & Startup */}
        <div className="flex flex-col gap-3">
          <div className="flex items-center gap-2 font-medium text-foreground/90">
            <Power className="w-4 h-4 text-primary" />
            <span>System Startup</span>
          </div>

          <label className="flex items-center justify-between p-2.5 rounded-lg border border-border/40 bg-card/40 cursor-pointer hover:bg-card/70 transition-colors">
            <div className="flex flex-col gap-0.5 pr-2">
              <span className="font-medium text-foreground/90">Launch at Windows Startup</span>
              <span className="text-[11px] text-muted-foreground leading-tight">
                Automatically start Clipo in the background when logging into Windows.
              </span>
            </div>
            <input
              type="checkbox"
              checked={launchAtStartup}
              onChange={(e) => setLaunchAtStartup(e.target.checked)}
              className="h-4 w-4 accent-primary rounded cursor-pointer"
            />
          </label>
        </div>

        {/* Danger Zone */}
        <div className="flex flex-col gap-3 pt-2 border-t border-border/40">
          <div className="flex items-center justify-between">
            <div className="flex flex-col gap-0.5">
              <span className="font-medium text-destructive">Clear History</span>
              <span className="text-[11px] text-muted-foreground">
                Permanently delete all unpinned clipboard clips.
              </span>
            </div>
            <Button
              type="button"
              variant="destructive"
              size="sm"
              className="h-7 px-3 text-xs gap-1.5"
              onClick={handleClearHistory}
            >
              <Trash2 className="w-3.5 h-3.5" />
              <span>Clear</span>
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}
