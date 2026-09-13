import clsx from "clsx";
import { AnimatePresence, motion } from "framer-motion";
import { AlertCircle, Check, ExternalLink, Loader2, PlayCircle } from "lucide-react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useAppStore } from "../store/useAppStore";
import { formatBytes } from "../lib/format";
import type { MediaItem } from "../types";

export function MediaCard({ item, index }: { item: MediaItem; index: number }) {
  const isSelected = useAppStore((s) => s.selected.has(item.id));
  const toggleSelect = useAppStore((s) => s.toggleSelect);
  const openPreview = useAppStore((s) => s.openPreview);
  const download = useAppStore((s) => s.downloads[item.id]);

  const pct =
    download?.total && download.total > 0
      ? Math.min(100, Math.round((download.downloaded / download.total) * 100))
      : null;

  return (
    <motion.div
      layout
      initial={{ opacity: 0, scale: 0.94 }}
      animate={{ opacity: 1, scale: 1 }}
      exit={{ opacity: 0, scale: 0.94 }}
      transition={{ duration: 0.18, delay: Math.min(index, 24) * 0.01 }}
      onClick={() => toggleSelect(item.id)}
      onDoubleClick={(e) => {
        e.stopPropagation();
        openPreview(item.id);
      }}
      title="Click to select · double-click to preview"
      className={clsx(
        "group relative aspect-square w-full cursor-pointer select-none overflow-hidden",
        isSelected ? "bg-accent-100" : "bg-surface-hover",
      )}
    >
      <motion.div
        animate={{
          scale: isSelected ? 0.86 : 1,
          borderRadius: isSelected ? 16 : 0,
        }}
        transition={{ duration: 0.16, ease: "easeOut" }}
        className="relative h-full w-full overflow-hidden"
      >
        <img
          src={item.thumbUrl}
          alt={item.originalName}
          loading="lazy"
          className="h-full w-full object-cover"
        />

        {/* hover scrim for checkbox / actions legibility */}
        <div className="pointer-events-none absolute inset-0 bg-gradient-to-b from-black/35 via-transparent to-transparent opacity-0 transition-opacity duration-150 group-hover:opacity-100" />
        <div className="pointer-events-none absolute inset-x-0 bottom-0 h-14 bg-gradient-to-t from-black/55 to-transparent opacity-0 transition-opacity duration-150 group-hover:opacity-100" />

        {item.isVideo && (
          <div className="absolute bottom-1.5 left-1.5 text-white drop-shadow-[0_1px_2px_rgba(0,0,0,0.6)]">
            <PlayCircle className="h-5 w-5" strokeWidth={1.75} fill="rgba(0,0,0,0.25)" />
          </div>
        )}

        <button
          onClick={(e) => {
            e.stopPropagation();
            void openUrl(item.mediaUrl);
          }}
          title="Open original"
          className="absolute right-1.5 top-1.5 flex h-7 w-7 items-center justify-center rounded-full text-white opacity-0 transition hover:bg-black/30 group-hover:opacity-100"
        >
          <ExternalLink className="h-3.5 w-3.5" />
        </button>

        <div className="pointer-events-none absolute inset-x-0 bottom-0 px-2 pb-1.5 pt-4 opacity-0 transition-opacity duration-150 group-hover:opacity-100">
          <p className="truncate text-[11px] font-medium text-white">{item.originalName}</p>
          <p className="text-[10px] text-white/75">
            {formatBytes(item.sizeBytes)}
            {item.width && item.height ? ` · ${item.width}×${item.height}` : ""}
          </p>
        </div>

        <AnimatePresence>
          {download && (
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              className="absolute inset-0 flex flex-col items-center justify-center gap-1 bg-black/55 backdrop-blur-sm"
            >
              {download.status === "downloading" || download.status === "queued" ? (
                <>
                  <Loader2 className="h-5 w-5 animate-spin text-white" />
                  {pct !== null && <span className="text-xs font-medium text-white">{pct}%</span>}
                </>
              ) : download.status === "done" ? (
                <motion.div
                  initial={{ scale: 0 }}
                  animate={{ scale: 1 }}
                  transition={{ type: "spring", stiffness: 400, damping: 18 }}
                  className="flex h-8 w-8 items-center justify-center rounded-full bg-success-500 text-white"
                >
                  <Check className="h-4 w-4" />
                </motion.div>
              ) : download.status === "error" ? (
                <div
                  title={download.error}
                  className="flex h-8 w-8 items-center justify-center rounded-full bg-danger-500 text-white"
                >
                  <AlertCircle className="h-4 w-4" />
                </div>
              ) : (
                <span className="text-xs font-medium text-white">Cancelled</span>
              )}
            </motion.div>
          )}
        </AnimatePresence>

        {download && (download.status === "downloading" || download.status === "queued") && pct !== null && (
          <div className="absolute inset-x-0 bottom-0 h-1 bg-black/30">
            <motion.div
              className="h-full bg-accent-500"
              animate={{ width: `${pct}%` }}
              transition={{ ease: "linear", duration: 0.15 }}
            />
          </div>
        )}
      </motion.div>

      <motion.div
        whileTap={{ scale: 0.85 }}
        className={clsx(
          "absolute left-2 top-2 flex h-5 w-5 items-center justify-center rounded-full border transition-all duration-150",
          isSelected
            ? "border-accent-500 bg-accent-500 text-white opacity-100"
            : "border-white bg-black/15 text-transparent opacity-0 backdrop-blur-sm group-hover:opacity-100",
        )}
      >
        <AnimatePresence>
          {isSelected && (
            <motion.span
              initial={{ scale: 0, opacity: 0 }}
              animate={{ scale: 1, opacity: 1 }}
              exit={{ scale: 0, opacity: 0 }}
              transition={{ duration: 0.12 }}
            >
              <Check className="h-3.5 w-3.5" />
            </motion.span>
          )}
        </AnimatePresence>
      </motion.div>
    </motion.div>
  );
}
