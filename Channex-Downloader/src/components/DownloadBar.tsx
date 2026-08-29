import { useMemo } from "react";
import clsx from "clsx";
import { motion } from "framer-motion";
import { AlertTriangle, CheckCheck, Download, FolderOpen, Settings, Square, X } from "lucide-react";
import { revealInFileManager } from "../lib/api";
import { computeThreadDownloadPath } from "../lib/path";
import { useAppStore } from "../store/useAppStore";

export function DownloadBar() {
  const thread = useAppStore((s) => s.thread);
  const selected = useAppStore((s) => s.selected);
  const deselectAll = useAppStore((s) => s.deselectAll);
  const downloadRoot = useAppStore((s) => s.downloadRoot);
  const openSettings = useAppStore((s) => s.openSettings);
  const downloads = useAppStore((s) => s.downloads);
  const isDownloading = useAppStore((s) => s.isDownloading);
  const downloadError = useAppStore((s) => s.downloadError);
  const lastDownloadPath = useAppStore((s) => s.lastDownloadPath);
  const startDownload = useAppStore((s) => s.startDownload);
  const cancelDownload = useAppStore((s) => s.cancelDownload);

  const stats = useMemo(() => {
    const entries = Object.values(downloads);
    const total = entries.length;
    const done = entries.filter((d) => d.status === "done").length;
    const errored = entries.filter((d) => d.status === "error").length;
    const cancelled = entries.filter((d) => d.status === "cancelled").length;
    const settled = done + errored + cancelled;
    return { total, done, errored, cancelled, settled };
  }, [downloads]);

  if (!thread) return null;

  const hasRun = stats.total > 0;
  const pct = stats.total > 0 ? Math.round((stats.settled / stats.total) * 100) : 0;
  const finishedCleanly = hasRun && !isDownloading && stats.settled === stats.total && !downloadError;
  const previewPath = downloadRoot
    ? computeThreadDownloadPath(downloadRoot, thread.board, thread.threadNo)
    : null;
  const canDownload = !!downloadRoot && !isDownloading;

  return (
    <div className="flex flex-col gap-3 border-t border-border bg-surface-alt px-6 py-4">
      {hasRun && (
        <div className="flex items-center gap-3">
          <div className="h-1.5 flex-1 overflow-hidden rounded-full bg-surface-sunken">
            <motion.div
              className={clsx("h-full rounded-full", stats.errored > 0 ? "bg-warning-500" : "bg-accent-500")}
              animate={{ width: `${pct}%` }}
              transition={{ duration: 0.25, ease: "easeOut" }}
            />
          </div>
          <span className="whitespace-nowrap text-xs text-text-secondary">
            {stats.settled} / {stats.total} files
            {stats.errored > 0 && ` · ${stats.errored} failed`}
          </span>
        </div>
      )}

      {downloadError && (
        <div className="flex items-center gap-2 rounded-lg border border-danger-500/30 bg-danger-100 px-3 py-2 text-xs text-danger-600">
          <AlertTriangle className="h-3.5 w-3.5 shrink-0" />
          {downloadError}
        </div>
      )}

      <div className="flex flex-wrap items-center gap-3">
        <button
          onClick={openSettings}
          className="flex min-w-0 max-w-md items-center gap-2 rounded-full border border-border bg-surface px-3 py-2 text-xs text-text-secondary hover:border-border-strong"
        >
          <FolderOpen className="h-3.5 w-3.5 shrink-0" />
          <span className="truncate font-mono">
            {previewPath ?? "Choose a download folder in Settings…"}
          </span>
          <Settings className="h-3 w-3 shrink-0 text-text-tertiary" />
        </button>

        <div className="ml-auto flex flex-wrap items-center gap-2">
          {finishedCleanly && lastDownloadPath && (
            <motion.button
              initial={{ opacity: 0, y: 4 }}
              animate={{ opacity: 1, y: 0 }}
              onClick={() => void revealInFileManager(lastDownloadPath)}
              className="flex items-center gap-2 rounded-full border border-border px-4 py-2.5 text-sm text-text-primary hover:border-border-strong hover:bg-surface-hover"
            >
              <FolderOpen className="h-4 w-4" />
              Show folder
            </motion.button>
          )}

          {isDownloading ? (
            <motion.button
              whileTap={{ scale: 0.96 }}
              onClick={() => void cancelDownload()}
              className="flex items-center gap-2 rounded-full bg-danger-500 px-4 py-2.5 text-sm font-medium text-white hover:bg-danger-600"
            >
              <Square className="h-3.5 w-3.5 fill-current" />
              Cancel
            </motion.button>
          ) : (
            <>
              <motion.button
                whileTap={{ scale: 0.96 }}
                onClick={deselectAll}
                disabled={selected.size === 0}
                className="flex items-center gap-2 rounded-full border border-border px-4 py-2.5 text-sm text-text-primary hover:border-border-strong hover:bg-surface-hover disabled:cursor-not-allowed disabled:text-text-tertiary disabled:hover:bg-transparent"
              >
                <X className="h-3.5 w-3.5" />
                Select none
              </motion.button>

              <motion.button
                whileTap={{ scale: 0.96 }}
                onClick={() => void startDownload("selected")}
                disabled={!canDownload || selected.size === 0}
                className="flex items-center gap-2 rounded-full border border-accent-500 px-4 py-2.5 text-sm font-medium text-accent-600 transition hover:bg-accent-100 disabled:cursor-not-allowed disabled:border-border disabled:text-text-tertiary disabled:hover:bg-transparent"
              >
                <Download className="h-4 w-4" />
                Download selected {selected.size > 0 ? `(${selected.size})` : ""}
              </motion.button>

              <motion.button
                whileTap={{ scale: 0.96 }}
                onClick={() => void startDownload("all")}
                disabled={!canDownload || thread.media.length === 0}
                className="flex items-center gap-2 rounded-full bg-accent-500 px-4 py-2.5 text-sm font-medium text-white shadow-sm transition hover:bg-accent-600 disabled:cursor-not-allowed disabled:bg-surface-sunken disabled:text-text-tertiary disabled:shadow-none"
              >
                <CheckCheck className="h-4 w-4" />
                Download all ({thread.media.length})
              </motion.button>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
