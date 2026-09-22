import { useState, useEffect, useRef } from "react";
import { useClipboard } from "@/shared/hooks/useClipboard";
import { SearchBar, CategoryTabs } from "@/widgets/clipboard/components";
import { ClipItem, EmptyState } from "@/features/clipboard/components";

export default function ClipboardPage() {
  const {
    clips,
    loading,
    searchQuery,
    setSearchQuery,
    activeCategory,
    setActiveCategory,
    pasteClip,
    pastePlain,
    copyClip,
    togglePin,
    deleteClip,
  } = useClipboard();

  const [selectedIndex, setSelectedIndex] = useState(0);
  const listRef = useRef<HTMLDivElement>(null);

  // Keep selected index within range when clips change
  useEffect(() => {
    if (selectedIndex >= clips.length) {
      setSelectedIndex(Math.max(0, clips.length - 1));
    }
  }, [clips.length, selectedIndex]);

  // Keyboard Navigation: Up/Down to navigate, Enter to quick paste, 1-9 for quick select
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ignore if user is actively typing in input and presses anything other than arrow keys or enter
      const isInput =
        document.activeElement?.tagName === "INPUT" ||
        document.activeElement?.tagName === "TEXTAREA";

      if (e.key === "ArrowDown") {
        e.preventDefault();
        setSelectedIndex((prev) => (prev < clips.length - 1 ? prev + 1 : prev));
      } else if (e.key === "ArrowUp") {
        e.preventDefault();
        setSelectedIndex((prev) => (prev > 0 ? prev - 1 : 0));
      } else if (e.key === "Enter" && !e.shiftKey) {
        if (clips[selectedIndex]) {
          e.preventDefault();
          pasteClip(clips[selectedIndex].id);
        }
      } else if (e.key === "Enter" && e.shiftKey) {
        if (clips[selectedIndex] && clips[selectedIndex].hasHtml) {
          e.preventDefault();
          pastePlain(clips[selectedIndex].id);
        }
      } else if (!isInput && e.key >= "1" && e.key <= "9") {
        const num = parseInt(e.key, 10) - 1;
        if (clips[num]) {
          e.preventDefault();
          pasteClip(clips[num].id);
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [clips, selectedIndex, pasteClip, pastePlain]);

  // Scroll selected item into view
  useEffect(() => {
    if (!listRef.current) return;
    const items = listRef.current.children;
    if (items[selectedIndex]) {
      (items[selectedIndex] as HTMLElement).scrollIntoView({
        block: "nearest",
        behavior: "smooth",
      });
    }
  }, [selectedIndex]);

  return (
    <div className="flex flex-col h-full bg-background">
      {/* Top Search & Category Filter Section */}
      <div className="flex flex-col gap-2 p-3 pb-2 border-b border-border/40 bg-card/20 flex-shrink-0">
        <SearchBar
          value={searchQuery}
          onChange={setSearchQuery}
          onClear={() => setSearchQuery("")}
        />
        <CategoryTabs active={activeCategory} onChange={setActiveCategory} />
      </div>

      {/* Clipboard List Scroll Area */}
      <div
        ref={listRef}
        className="flex-1 overflow-y-auto no-scrollbar p-3 space-y-2 select-none"
      >
        {loading ? (
          <div className="flex items-center justify-center h-48 text-muted-foreground text-xs">
            Loading clips...
          </div>
        ) : clips.length === 0 ? (
          <EmptyState
            isSearch={Boolean(searchQuery.trim()) || activeCategory !== "all"}
            query={searchQuery}
          />
        ) : (
          clips.map((clip, index) => (
            <ClipItem
              key={clip.id}
              clip={clip}
              index={index}
              isSelected={index === selectedIndex}
              onSelect={(c) => {
                setSelectedIndex(index);
                pasteClip(c.id);
              }}
              onCopy={(c) => copyClip(c.id)}
              onPastePlain={(c) => pastePlain(c.id)}
              onTogglePin={(c) => togglePin(c.id)}
              onDelete={(c) => deleteClip(c.id)}
            />
          ))
        )}
      </div>

      {/* Bottom Hint Footer */}
      <footer className="px-3 py-1.5 border-t border-border/40 bg-card/40 flex items-center justify-between text-[10px] text-muted-foreground font-mono flex-shrink-0">
        <div className="flex items-center gap-2">
          <span>
            <kbd className="px-1 py-0.5 bg-muted rounded border border-border/40">↑↓</kbd> navigate
          </span>
          <span>
            <kbd className="px-1 py-0.5 bg-muted rounded border border-border/40">↵</kbd> paste
          </span>
          <span>
            <kbd className="px-1 py-0.5 bg-muted rounded border border-border/40">⇧↵</kbd> plain
          </span>
        </div>
        <div>
          <span>{clips.length} {clips.length === 1 ? "clip" : "clips"}</span>
        </div>
      </footer>
    </div>
  );
}
