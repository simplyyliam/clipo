import { Button } from "@/components/ui/button";
import { Copy, Pin, PinOff, Trash2, ClipboardType } from "lucide-react";

interface QuickActionsProps {
  pinned: boolean;
  hasHtml?: boolean;
  onCopy: (e: React.MouseEvent) => void;
  onPastePlain?: (e: React.MouseEvent) => void;
  onTogglePin: (e: React.MouseEvent) => void;
  onDelete: (e: React.MouseEvent) => void;
}

export function QuickActions({
  pinned,
  hasHtml,
  onCopy,
  onPastePlain,
  onTogglePin,
  onDelete,
}: QuickActionsProps) {
  return (
    <div
      className="flex items-center gap-0.5 bg-background/90 backdrop-blur-sm border border-border/60 rounded-md p-0.5 shadow-sm"
      onClick={(e) => e.stopPropagation()}
    >
      <Button
        variant="ghost"
        size="icon"
        className="h-6 w-6 text-muted-foreground hover:text-foreground"
        title="Copy to clipboard"
        onClick={onCopy}
      >
        <Copy className="w-3.5 h-3.5" />
      </Button>

      {hasHtml && onPastePlain && (
        <Button
          variant="ghost"
          size="icon"
          className="h-6 w-6 text-muted-foreground hover:text-foreground"
          title="Paste as plain text"
          onClick={onPastePlain}
        >
          <ClipboardType className="w-3.5 h-3.5" />
        </Button>
      )}

      <Button
        variant="ghost"
        size="icon"
        className={`h-6 w-6 ${pinned ? "text-amber-500 hover:text-amber-600" : "text-muted-foreground hover:text-foreground"}`}
        title={pinned ? "Unpin" : "Pin to top"}
        onClick={onTogglePin}
      >
        {pinned ? <PinOff className="w-3.5 h-3.5" /> : <Pin className="w-3.5 h-3.5" />}
      </Button>

      <Button
        variant="ghost"
        size="icon"
        className="h-6 w-6 text-muted-foreground hover:text-destructive"
        title="Delete clip"
        onClick={onDelete}
      >
        <Trash2 className="w-3.5 h-3.5" />
      </Button>
    </div>
  );
}
