import type { FilterCategory } from "@/shared/types";
import { Pin, FileText, Image as ImageIcon, Link as LinkIcon, Files, Layers } from "lucide-react";

interface CategoryTabsProps {
  active: FilterCategory;
  onChange: (cat: FilterCategory) => void;
}

const CATEGORIES: { id: FilterCategory; label: string; icon: React.ComponentType<{ className?: string }> }[] = [
  { id: "all", label: "All", icon: Layers },
  { id: "pinned", label: "Pinned", icon: Pin },
  { id: "text", label: "Text", icon: FileText },
  { id: "image", label: "Images", icon: ImageIcon },
  { id: "link", label: "Links", icon: LinkIcon },
  { id: "files", label: "Files", icon: Files },
];

export function CategoryTabs({ active, onChange }: CategoryTabsProps) {
  return (
    <div className="flex items-center gap-1 overflow-x-auto no-scrollbar py-1 text-xs">
      {CATEGORIES.map((cat) => {
        const Icon = cat.icon;
        const isActive = active === cat.id;
        return (
          <button
            key={cat.id}
            type="button"
            onClick={() => onChange(cat.id)}
            className={`flex items-center gap-1.5 px-2.5 py-1 rounded-md font-medium transition-all whitespace-nowrap select-none ${
              isActive
                ? "bg-primary text-primary-foreground shadow-xs"
                : "text-muted-foreground hover:text-foreground hover:bg-muted/60"
            }`}
          >
            <Icon className="w-3 h-3" />
            <span>{cat.label}</span>
          </button>
        );
      })}
    </div>
  );
}
