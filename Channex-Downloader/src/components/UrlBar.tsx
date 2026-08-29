import { Loader2, Search, X } from "lucide-react";
import { AnimatePresence, motion } from "framer-motion";
import { useAppStore } from "../store/useAppStore";
import { SITE_PRESETS } from "../types";

export function UrlBar() {
  const url = useAppStore((s) => s.url);
  const setUrl = useAppStore((s) => s.setUrl);
  const fetchThread = useAppStore((s) => s.fetchThread);
  const clearThread = useAppStore((s) => s.clearThread);
  const loading = useAppStore((s) => s.loading);
  const thread = useAppStore((s) => s.thread);
  const preferredSite = useAppStore((s) => s.preferredSite);

  const preset = SITE_PRESETS.find((p) => p.id === preferredSite) ?? SITE_PRESETS[0];
  const canClear = !!thread || !!url;

  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        void fetchThread();
      }}
      className="flex items-center gap-2"
    >
      <div className="relative flex-1 max-w-2xl">
        <Search className="pointer-events-none absolute left-4 top-1/2 h-4 w-4 -translate-y-1/2 text-text-tertiary" />
        <input
          value={url}
          onChange={(e) => setUrl(e.target.value)}
          placeholder={`Paste a thread link — e.g. ${preset.example}`}
          spellCheck={false}
          className="w-full rounded-full border border-transparent bg-surface-hover py-2.5 pl-11 pr-4 text-sm text-text-primary placeholder:text-text-tertiary outline-none transition focus:border-border focus:bg-surface focus:shadow-[0_1px_6px_rgba(32,33,36,0.18)]"
        />

        <AnimatePresence>
          {canClear && (
            <motion.button
              key="clear"
              type="button"
              initial={{ opacity: 0, scale: 0.9 }}
              animate={{ opacity: 1, scale: 1 }}
              exit={{ opacity: 0, scale: 0.9 }}
              whileTap={{ scale: 0.9 }}
              onClick={clearThread}
              title="Clear the loaded thread"
              className="absolute right-2 top-1/2 flex h-7 w-7 -translate-y-1/2 items-center justify-center rounded-full text-text-tertiary hover:bg-surface-sunken hover:text-text-primary"
            >
              <X className="h-4 w-4" />
            </motion.button>
          )}
        </AnimatePresence>
      </div>

      <motion.button
        type="submit"
        whileTap={{ scale: 0.96 }}
        disabled={loading || !url.trim()}
        className="flex items-center gap-2 rounded-full bg-accent-500 px-5 py-2.5 text-sm font-medium text-white shadow-sm transition hover:bg-accent-600 disabled:cursor-not-allowed disabled:bg-surface-sunken disabled:text-text-tertiary disabled:shadow-none"
      >
        {loading && <Loader2 className="h-4 w-4 animate-spin" />}
        {loading ? "Fetching…" : "Fetch thread"}
      </motion.button>
    </form>
  );
}
