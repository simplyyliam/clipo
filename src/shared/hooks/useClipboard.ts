import { useState, useEffect, useCallback } from "react";
import type { ClipView, FilterCategory } from "../types";
import { api } from "../ipc/api";

export function useClipboard() {
  const [clips, setClips] = useState<ClipView[]>([]);
  const [loading, setLoading] = useState(true);
  const [searchQuery, setSearchQuery] = useState("");
  const [activeCategory, setActiveCategory] = useState<FilterCategory>("all");

  const refresh = useCallback(async () => {
    try {
      const data = await api.getClips();
      setClips(data);
    } catch (err) {
      console.error("Failed to load clips:", err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
    const unlistenPromise = api.onClipsChanged(() => {
      refresh();
    });

    return () => {
      unlistenPromise.then((unlisten) => unlisten());
    };
  }, [refresh]);

  const filteredClips = clips.filter((clip) => {
    // 1. Category Filter
    if (activeCategory === "pinned" && !clip.pinned) return false;
    if (activeCategory === "text" && clip.kind !== "text") return false;
    if (activeCategory === "image" && clip.kind !== "image") return false;
    if (activeCategory === "link" && clip.kind !== "link") return false;
    if (activeCategory === "files" && clip.kind !== "files") return false;

    // 2. Search Query Filter
    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase().trim();
      return clip.searchText.includes(q) || clip.preview.toLowerCase().includes(q);
    }

    return true;
  });

  return {
    clips: filteredClips,
    rawClips: clips,
    loading,
    searchQuery,
    setSearchQuery,
    activeCategory,
    setActiveCategory,
    refresh,
    copyClip: api.copyClip,
    pasteClip: api.pasteClip,
    pastePlain: api.pastePlain,
    togglePin: api.togglePin,
    deleteClip: api.deleteClip,
    clearHistory: api.clearHistory,
  };
}
