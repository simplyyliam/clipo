import type { ClipView } from "@/shared/types";
import { AppIcon } from "./AppIcon";
import { ClipThumbnail } from "./ClipThumbnail";
import { QuickActions } from "./QuickActions";
import { Pin, FileText, Image as ImageIcon, Link as LinkIcon, Files, ExternalLink } from "lucide-react";

interface ClipItemProps {
  clip: ClipView;
  index: number;
  isSelected?: boolean;
  onSelect: (clip: ClipView) => void;
  onCopy: (clip: ClipView) => void;
  onPastePlain?: (clip: ClipView) => void;
  onTogglePin: (clip: ClipView) => void;
  onDelete: (clip: ClipView) => void;
}

export function ClipItem({
  clip,
  index,
  isSelected,
  onSelect,
  onCopy,
  onPastePlain,
  onTogglePin,
  onDelete,
}: ClipItemProps) {
  const getKindIcon = () => {
    switch (clip.kind) {
      case "image":
        return <ImageIcon className="w-3.5 h-3.5 text-blue-500" />;
      case "link":
        return <LinkIcon className="w-3.5 h-3.5 text-emerald-500" />;
      case "files":
        return <Files className="w-3.5 h-3.5 text-amber-500" />;
      default:
        return <FileText className="w-3.5 h-3.5 text-muted-foreground" />;
    }
  };

  const formatTimestamp = (epochSecs: number) => {
    const now = Math.floor(Date.now() / 1000);
    const diff = now - epochSecs;
    if (diff < 60) return "Just now";
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
    return new Date(epochSecs * 1000).toLocaleDateString(undefined, {
      month: "short",
      day: "numeric",
    });
  };

  return (
    <div
      onClick={() => onSelect(clip)}
      className={`group relative flex flex-col gap-2 p-3 rounded-xl border transition-all cursor-pointer select-none text-left ${
        isSelected
          ? "bg-accent/80 border-primary/50 shadow-sm"
          : "bg-card/70 hover:bg-accent/40 border-border/40 hover:border-border/80"
      }`}
    >
      {/* Header Info */}
      <div className="flex items-center justify-between gap-2 text-xs text-muted-foreground">
        <div className="flex items-center gap-1.5 min-w-0">
          <AppIcon source={clip.source} className="w-3.5 h-3.5 flex-shrink-0" />
          <span className="truncate max-w-[120px] font-medium text-foreground/80">
            {clip.source?.name || "Clipboard"}
          </span>
          <span className="text-border">·</span>
          <span className="flex items-center gap-1 flex-shrink-0">
            {getKindIcon()}
            <span className="capitalize">{clip.kind}</span>
          </span>
        </div>

        <div className="flex items-center gap-1.5 flex-shrink-0">
          {clip.pinned && <Pin className="w-3 h-3 text-amber-500 fill-amber-500" />}
          {index < 9 && (
            <kbd className="hidden group-hover:inline-block px-1 py-0.5 text-[10px] font-mono bg-muted text-muted-foreground rounded border border-border/50">
              #{index + 1}
            </kbd>
          )}
          <span>{formatTimestamp(clip.createdAt)}</span>
        </div>
      </div>

      {/* Content Preview */}
      <div className="text-sm font-normal overflow-hidden">
        {clip.kind === "image" ? (
          <ClipThumbnail id={clip.id} alt={clip.preview} className="w-full h-28 my-1" />
        ) : clip.kind === "files" ? (
          <div className="flex flex-col gap-1 py-0.5">
            <div className="flex items-center gap-1.5 text-xs text-foreground font-mono truncate">
              <Files className="w-3.5 h-3.5 text-amber-500 flex-shrink-0" />
              <span className="truncate">{clip.preview}</span>
            </div>
          </div>
        ) : clip.kind === "link" ? (
          <div className="flex items-start gap-1.5 text-primary text-xs font-mono break-all line-clamp-3">
            <ExternalLink className="w-3.5 h-3.5 mt-0.5 flex-shrink-0 opacity-70" />
            <span>{clip.preview}</span>
          </div>
        ) : (
          <p className="text-xs text-foreground/90 whitespace-pre-wrap break-words line-clamp-3 font-sans leading-relaxed">
            {clip.preview}
          </p>
        )}
      </div>

      {/* Quick Action Overlay on hover/focus */}
      <div className="absolute top-2 right-2 opacity-0 group-hover:opacity-100 focus-within:opacity-100 transition-opacity">
        <QuickActions
          pinned={clip.pinned}
          hasHtml={clip.hasHtml}
          onCopy={(e) => {
            e.stopPropagation();
            onCopy(clip);
          }}
          onPastePlain={
            clip.hasHtml && onPastePlain
              ? (e) => {
                  e.stopPropagation();
                  onPastePlain(clip);
                }
              : undefined
          }
          onTogglePin={(e) => {
            e.stopPropagation();
            onTogglePin(clip);
          }}
          onDelete={(e) => {
            e.stopPropagation();
            onDelete(clip);
          }}
        />
      </div>
    </div>
  );
}
