import { useEffect, useMemo } from "react";
import clsx from "clsx";
import { AnimatePresence, motion } from "framer-motion";
import { openUrl } from "@tauri-apps/plugin-opener";
import {
  AlertCircle,
  Check,
  ChevronLeft,
  ChevronRight,
  Download,
  ExternalLink,
  Loader2,
  X,
} from "lucide-react";
import { useShallow } from "zustand/react/shallow";
import { formatBytes } from "../lib/format";
import { useMediaBlob } from "../hooks/useMediaBlob";
import { selectVisibleMedia, useAppStore } from "../store/useAppStore";

export function PreviewModal() {
  const previewId = useAppStore((s) => s.previewId);
  const closePreview = useAppStore((s) => s.closePreview);
  const previewNext = useAppStore((s) => s.previewNext);
  const previewPrev = useAppStore((s) => s.previewPrev);
  const openPreview = useAppStore((s) => s.openPreview);
  const downloadSingle = useAppStore((s) => s.downloadSingle);
  const isDownloading = useAppStore((s) => s.isDownloading);
  const visible = useAppStore(useShallow(selectVisibleMedia));
  const download = useAppStore((s) => (previewId ? s.downloads[previewId] : undefined));

  const item = useMemo(() => visible.find((m) => m.id === previewId) ?? null, [visible, previewId]);
  const {
    url: blobUrl,
    loading: mediaLoading,
    error: mediaError,
  } = useMediaBlob(item?.mediaUrl ?? null, item?.ext ?? null);

  useEffect(() => {
    if (!previewId) return;
    function onKey(e: KeyboardEvent) {
      if (e.key === "Escape") closePreview();
      else if (e.key === "ArrowRight") previewNext();
      else if (e.key === "ArrowLeft") previewPrev();
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [previewId, closePreview, previewNext, previewPrev]);

  const pct =
    download?.total && download.total > 0
      ? Math.min(100, Math.round((download.downloaded / download.total) * 100))
      : null;

  return (
    <AnimatePresence>
      {item && (
        <motion.div
          key="preview"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          transition={{ duration: 0.18 }}
          onClick={closePreview}
          className="fixed inset-0 z-[60] flex flex-col bg-black/90 backdrop-blur-md"
        >
          <div
            onClick={(e) => e.stopPropagation()}
            className="flex items-center justify-between gap-4 px-6 py-4"
          >
            <div className="min-w-0">
              <p className="truncate text-sm font-medium text-white">{item.originalName}</p>
              <p className="text-xs text-white/60">
                {formatBytes(item.sizeBytes)}
                {item.width && item.height ? ` · ${item.width}×${item.height}` : ""}
              </p>
            </div>

            <div className="flex shrink-0 items-center gap-2">
              <button
                onClick={() => void openUrl(item.mediaUrl)}
                title="Open original in browser"
                className="flex h-9 w-9 items-center justify-center rounded-full text-white/80 hover:bg-white/10 hover:text-white"
              >
                <ExternalLink className="h-4 w-4" />
              </button>

              <motion.button
                whileTap={{ scale: 0.96 }}
                onClick={() => void downloadSingle(item.id)}
                disabled={isDownloading}
                title="Download this file"
                className="flex items-center gap-2 rounded-full bg-accent-500 px-3.5 py-2 text-sm font-medium text-white hover:bg-accent-600 disabled:cursor-not-allowed disabled:bg-white/10 disabled:text-white/40"
              >
                {download?.status === "downloading" || download?.status === "queued" ? (
                  <>
                    <Loader2 className="h-4 w-4 animate-spin" />
                    {pct !== null ? `${pct}%` : "Downloading…"}
                  </>
                ) : download?.status === "done" ? (
                  <>
                    <Check className="h-4 w-4" />
                    Downloaded
                  </>
                ) : download?.status === "error" ? (
                  <>
                    <AlertCircle className="h-4 w-4" />
                    Retry
                  </>
                ) : (
                  <>
                    <Download className="h-4 w-4" />
                    Download
                  </>
                )}
              </motion.button>

              <button
                onClick={closePreview}
                title="Close"
                className="flex h-9 w-9 items-center justify-center rounded-full text-white/80 hover:bg-white/10 hover:text-white"
              >
                <X className="h-4 w-4" />
              </button>
            </div>
          </div>

          <div
            onClick={(e) => e.stopPropagation()}
            className="relative flex flex-1 items-center justify-center overflow-hidden px-6"
          >
            {visible.length > 1 && (
              <button
                onClick={previewPrev}
                className="absolute left-4 flex h-11 w-11 items-center justify-center rounded-full bg-black/40 text-white backdrop-blur transition hover:bg-black/60"
              >
                <ChevronLeft className="h-6 w-6" />
              </button>
            )}

            <AnimatePresence mode="wait">
              {mediaLoading ? (
                <motion.div
                  key={`loading-${item.id}`}
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  exit={{ opacity: 0 }}
                  className="flex flex-col items-center gap-3 text-white/60"
                >
                  <Loader2 className="h-8 w-8 animate-spin" />
                  <span className="text-sm">Loading preview…</span>
                </motion.div>
              ) : mediaError ? (
                <motion.div
                  key={`error-${item.id}`}
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  exit={{ opacity: 0 }}
                  className="flex flex-col items-center gap-3 text-center"
                >
                  <AlertCircle className="h-8 w-8 text-danger-500" />
                  <p className="max-w-sm text-sm text-white/70">{mediaError}</p>
                  <button
                    onClick={() => void openUrl(item.mediaUrl)}
                    className="flex items-center gap-2 rounded-full border border-white/20 px-3 py-1.5 text-xs text-white/80 hover:border-white/40"
                  >
                    <ExternalLink className="h-3.5 w-3.5" />
                    Open in browser instead
                  </button>
                </motion.div>
              ) : blobUrl && item.isVideo ? (
                <motion.video
                  key={item.id}
                  initial={{ opacity: 0, scale: 0.98 }}
                  animate={{ opacity: 1, scale: 1 }}
                  exit={{ opacity: 0 }}
                  transition={{ duration: 0.15 }}
                  src={blobUrl}
                  controls
                  autoPlay
                  loop
                  className="max-h-full max-w-full rounded-lg shadow-2xl"
                />
              ) : blobUrl ? (
                <motion.img
                  key={item.id}
                  initial={{ opacity: 0, scale: 0.98 }}
                  animate={{ opacity: 1, scale: 1 }}
                  exit={{ opacity: 0 }}
                  transition={{ duration: 0.15 }}
                  src={blobUrl}
                  alt={item.originalName}
                  className="max-h-full max-w-full rounded-lg object-contain shadow-2xl"
                />
              ) : null}
            </AnimatePresence>

            {visible.length > 1 && (
              <button
                onClick={previewNext}
                className="absolute right-4 flex h-11 w-11 items-center justify-center rounded-full bg-black/40 text-white backdrop-blur transition hover:bg-black/60"
              >
                <ChevronRight className="h-6 w-6" />
              </button>
            )}
          </div>

          {visible.length > 1 && (
            <div
              onClick={(e) => e.stopPropagation()}
              className="flex gap-2 overflow-x-auto px-6 py-4"
            >
              {visible.map((m) => (
                <button
                  key={m.id}
                  onClick={() => openPreview(m.id)}
                  className={clsx(
                    "relative h-16 w-16 shrink-0 overflow-hidden rounded-lg border-2 transition",
                    m.id === item.id
                      ? "border-accent-300"
                      : "border-transparent opacity-50 hover:opacity-100",
                  )}
                >
                  <img src={m.thumbUrl} alt={m.originalName} className="h-full w-full object-cover" />
                </button>
              ))}
            </div>
          )}
        </motion.div>
      )}
    </AnimatePresence>
  );
}
