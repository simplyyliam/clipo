import { Outlet, useLocation, useNavigate } from "react-router-dom";
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
import { Settings as SettingsIcon, Layers } from "lucide-react";
import { Button } from "@/components/ui/button";

export function AppLayout() {
  const location = useLocation();
  const navigate = useNavigate();

  const isSettings = location.pathname.startsWith("/settings");

  const handleHeaderMouseDown = (e: React.MouseEvent) => {
    // Only trigger drag if left click and not clicking a button/interactive element
    if (e.button === 0 && !(e.target as HTMLElement).closest("button, a, input")) {
      getCurrentWebviewWindow().startDragging();
    }
  };

  return (
    <div className="flex flex-col h-screen w-screen bg-background/95 backdrop-blur-xl text-foreground select-none overflow-hidden rounded-2xl border border-border/80 shadow-2xl">
      {/* Top Drag & Navigation Bar */}
      <header
        onMouseDown={handleHeaderMouseDown}
        className="flex items-center justify-between px-3.5 py-2.5 border-b border-border/50 bg-card/50 shrink-0 cursor-grab active:cursor-grabbing select-none"
      >
        <div className="flex items-center gap-2 pointer-events-none select-none">
          <div className="w-5 h-5 rounded-md bg-primary flex items-center justify-center text-primary-foreground font-bold text-xs shadow-xs">
            C
          </div>
          <span className="font-semibold text-xs tracking-tight text-foreground/90">
            Clipo
          </span>
        </div>

        <div className="flex items-center gap-1">
          <Button
            variant="ghost"
            size="icon"
            className={`h-7 w-7 rounded-md transition-colors ${
              !isSettings
                ? "text-primary bg-primary/10"
                : "text-muted-foreground hover:text-foreground hover:bg-muted/60"
            }`}
            title="Clipboard History"
            onClick={() => navigate("/")}
          >
            <Layers className="w-3.5 h-3.5" />
          </Button>

          <Button
            variant="ghost"
            size="icon"
            className={`h-7 w-7 rounded-md transition-colors ${
              isSettings
                ? "text-primary bg-primary/10"
                : "text-muted-foreground hover:text-foreground hover:bg-muted/60"
            }`}
            title="Preferences"
            onClick={() => navigate("/settings")}
          >
            <SettingsIcon className="w-3.5 h-3.5" />
          </Button>
        </div>
      </header>

      {/* Main Content View with Nested Route Outlet */}
      <main className="flex-1 overflow-hidden relative">
        <Outlet />
      </main>
    </div>
  );
}

export default AppLayout;
