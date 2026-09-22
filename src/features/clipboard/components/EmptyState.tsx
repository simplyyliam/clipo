import { ClipboardList, SearchX } from "lucide-react";

interface EmptyStateProps {
  isSearch?: boolean;
  query?: string;
}

export function EmptyState({ isSearch, query }: EmptyStateProps) {
  if (isSearch) {
    return (
      <div className="flex flex-col items-center justify-center h-64 p-6 text-center text-muted-foreground animate-fade-in">
        <div className="w-12 h-12 rounded-full bg-muted/30 flex items-center justify-center mb-3 border border-border/40">
          <SearchX className="w-6 h-6 text-muted-foreground/60" />
        </div>
        <p className="text-sm font-medium text-foreground">No matching clips found</p>
        <p className="text-xs mt-1 text-muted-foreground max-w-xs">
          No items match &quot;{query}&quot;. Try adjusting your search query or filter category.
        </p>
      </div>
    );
  }

  return (
    <div className="flex flex-col items-center justify-center h-64 p-6 text-center text-muted-foreground animate-fade-in">
      <div className="w-12 h-12 rounded-full bg-muted/30 flex items-center justify-center mb-3 border border-border/40">
        <ClipboardList className="w-6 h-6 text-muted-foreground/60" />
      </div>
      <p className="text-sm font-medium text-foreground">No clipboard history yet</p>
      <p className="text-xs mt-1 text-muted-foreground max-w-xs">
        Copy anything with <kbd className="px-1 py-0.5 text-[10px] font-mono bg-muted rounded border border-border/50">Ctrl+C</kbd> and it will show up here automatically.
      </p>
    </div>
  );
}
