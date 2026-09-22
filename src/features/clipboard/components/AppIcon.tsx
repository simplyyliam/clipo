import { useState, useEffect } from "react";
import type { SourceApp } from "@/shared/types";
import { api } from "@/shared/ipc/api";
import { AppWindow } from "lucide-react";

interface AppIconProps {
  source?: SourceApp;
  className?: string;
}

export function AppIcon({ source, className = "w-4 h-4" }: AppIconProps) {
  const [iconBase64, setIconBase64] = useState<string | null>(null);

  useEffect(() => {
    if (!source?.exeName) return;
    api.getAppIcon(source.exeName, source.exePath).then((b64) => {
      if (b64) setIconBase64(b64);
    });
  }, [source?.exeName, source?.exePath]);

  if (iconBase64) {
    return (
      <img
        src={`data:image/png;base64,${iconBase64}`}
        alt={source?.name || "App"}
        className={`${className} object-contain rounded-sm`}
      />
    );
  }

  return <AppWindow className={`${className} text-muted-foreground/60`} />;
}
