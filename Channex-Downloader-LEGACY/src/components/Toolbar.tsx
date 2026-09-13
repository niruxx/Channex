import clsx from "clsx";
import { motion } from "framer-motion";
import { Search } from "lucide-react";
import { useShallow } from "zustand/react/shallow";
import { useAppStore, selectVisibleMedia } from "../store/useAppStore";
import type { MediaFilter } from "../types";

const FILTERS: { key: MediaFilter; label: string }[] = [
  { key: "all", label: "All" },
  { key: "images", label: "Images" },
  { key: "videos", label: "Videos" },
];

export function Toolbar() {
  const filter = useAppStore((s) => s.filter);
  const setFilter = useAppStore((s) => s.setFilter);
  const search = useAppStore((s) => s.search);
  const setSearch = useAppStore((s) => s.setSearch);
  const selected = useAppStore((s) => s.selected);
  const selectAll = useAppStore((s) => s.selectAll);
  const deselectAll = useAppStore((s) => s.deselectAll);
  const visible = useAppStore(useShallow(selectVisibleMedia));

  const visibleSelectedCount = visible.filter((m) => selected.has(m.id)).length;

  return (
    <div className="flex flex-wrap items-center gap-3 py-3">
      <div className="flex items-center gap-1">
        {FILTERS.map((f) => (
          <button
            key={f.key}
            onClick={() => setFilter(f.key)}
            className={clsx(
              "relative rounded-full px-3.5 py-1.5 text-xs font-medium transition-colors",
              filter === f.key ? "text-accent-600" : "text-text-secondary hover:bg-surface-hover",
            )}
          >
            {filter === f.key && (
              <motion.div
                layoutId="filter-pill"
                transition={{ duration: 0.18, ease: "easeOut" }}
                className="absolute inset-0 rounded-full bg-accent-100"
              />
            )}
            <span className="relative">{f.label}</span>
          </button>
        ))}
      </div>

      <div className="relative">
        <Search className="pointer-events-none absolute left-2.5 top-1/2 h-3.5 w-3.5 -translate-y-1/2 text-text-tertiary" />
        <input
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          placeholder="Filter filenames…"
          spellCheck={false}
          className="w-48 rounded-full border border-transparent bg-surface-hover py-1.5 pl-7 pr-3 text-xs text-text-primary placeholder:text-text-tertiary outline-none focus:border-border focus:bg-surface"
        />
      </div>

      <div className="ml-auto flex items-center gap-3 text-xs text-text-secondary">
        <span>
          {visibleSelectedCount} / {visible.length} selected
        </span>
        <button onClick={selectAll} className="font-medium text-accent-500 hover:text-accent-600">
          Select all
        </button>
        <button onClick={deselectAll} className="text-text-secondary hover:text-text-primary">
          Clear
        </button>
      </div>
    </div>
  );
}
