import { useState, useEffect } from "react";
import { api } from "@/shared/ipc/api";
import { ImageIcon } from "lucide-react";

interface ClipThumbnailProps {
  id: string;
  alt?: string;
  className?: string;
}

export function ClipThumbnail({ id, alt = "Preview", className = "w-full h-32" }: ClipThumbnailProps) {
  const [src, setSrc] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let mounted = true;
    api.getClipThumbnail(id).then((b64) => {
      if (!mounted) return;
      if (b64) {
        setSrc(`data:image/png;base64,${b64}`);
      }
      setLoading(false);
    });

    return () => {
      mounted = false;
    };
  }, [id]);

  if (loading) {
    return (
      <div className={`${className} bg-muted/40 animate-pulse rounded-lg flex items-center justify-center`}>
        <ImageIcon className="w-6 h-6 text-muted-foreground/30" />
      </div>
    );
  }

  if (!src) {
    return (
      <div className={`${className} bg-muted/20 rounded-lg flex items-center justify-center text-xs text-muted-foreground`}>
        Image not available
      </div>
    );
  }

  return (
    <img
      src={src}
      alt={alt}
      className={`${className} object-contain rounded-lg border border-border/40`}
    />
  );
}
