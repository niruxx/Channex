import { AnimatePresence, motion } from "framer-motion";
import { openUrl } from "@tauri-apps/plugin-opener";
import { ExternalLink, FolderOpen, Monitor, Moon, Sun, X } from "lucide-react";
import clsx from "clsx";
import { useAppStore } from "../store/useAppStore";
import { SITE_PRESETS } from "../types";
import type { ThemePref } from "../lib/theme";

const CONCURRENCY_OPTIONS = [2, 4, 6, 8, 12, 16];

const THEME_OPTIONS: { key: ThemePref; label: string; icon: typeof Sun }[] = [
  { key: "system", label: "System", icon: Monitor },
  { key: "light", label: "Light", icon: Sun },
  { key: "dark", label: "Dark", icon: Moon },
];

export function SettingsPanel() {
  const open = useAppStore((s) => s.settingsOpen);
  const close = useAppStore((s) => s.closeSettings);
  const preferredSite = useAppStore((s) => s.preferredSite);
  const setPreferredSite = useAppStore((s) => s.setPreferredSite);
  const customOrigin = useAppStore((s) => s.customOrigin);
  const setCustomOrigin = useAppStore((s) => s.setCustomOrigin);
  const downloadRoot = useAppStore((s) => s.downloadRoot);
  const pickDownloadRoot = useAppStore((s) => s.pickDownloadRoot);
  const concurrency = useAppStore((s) => s.concurrency);
  const setConcurrency = useAppStore((s) => s.setConcurrency);
  const theme = useAppStore((s) => s.theme);
  const setTheme = useAppStore((s) => s.setTheme);

  const activePreset = SITE_PRESETS.find((p) => p.id === preferredSite) ?? SITE_PRESETS[0];
  const activeOrigin = activePreset.id === "custom" ? customOrigin.trim() : activePreset.origin;

  return (
    <AnimatePresence>
      {open && (
        <>
          <motion.div
            key="backdrop"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.15 }}
            onClick={close}
            className="fixed inset-0 z-40 bg-black/30"
          />
          <motion.div
            key="panel"
            initial={{ x: "100%" }}
            animate={{ x: 0 }}
            exit={{ x: "100%" }}
            transition={{ duration: 0.22, ease: "easeOut" }}
            className="fixed right-0 top-0 z-50 flex h-full w-full max-w-sm flex-col border-l border-border bg-surface shadow-2xl"
          >
            <div className="flex items-center justify-between border-b border-border px-5 py-4">
              <h2 className="text-sm font-medium text-text-primary">Settings</h2>
              <button
                onClick={close}
                className="flex h-7 w-7 items-center justify-center rounded-full text-text-secondary hover:bg-surface-hover hover:text-text-primary"
              >
                <X className="h-4 w-4" />
              </button>
            </div>

            <div className="flex-1 overflow-y-auto px-5 py-5">
              <section className="mb-8">
                <h3 className="mb-1 text-xs font-medium uppercase tracking-wide text-text-tertiary">
                  Appearance
                </h3>
                <p className="mb-3 text-xs text-text-secondary">Choose how Channex Downloader looks.</p>
                <div className="grid grid-cols-3 gap-2">
                  {THEME_OPTIONS.map(({ key, label, icon: Icon }) => (
                    <button
                      key={key}
                      onClick={() => setTheme(key)}
                      className={clsx(
                        "flex flex-col items-center gap-1.5 rounded-lg border px-3 py-2.5 text-xs font-medium transition",
                        theme === key
                          ? "border-accent-500 bg-accent-100 text-accent-600"
                          : "border-border bg-surface text-text-secondary hover:border-border-strong",
                      )}
                    >
                      <Icon className="h-4 w-4" />
                      {label}
                    </button>
                  ))}
                </div>
              </section>

              <section className="mb-8">
                <h3 className="mb-1 text-xs font-medium uppercase tracking-wide text-text-tertiary">
                  Imageboard
                </h3>
                <p className="mb-3 text-xs text-text-secondary">
                  Pick the site you browse most. It tailors the paste-a-link hint and gives you a
                  quick way to open the board list.
                </p>
                <div className="grid grid-cols-2 gap-2">
                  {SITE_PRESETS.map((preset) => (
                    <button
                      key={preset.id}
                      onClick={() => setPreferredSite(preset.id)}
                      className={clsx(
                        "rounded-lg border px-3 py-2 text-left text-xs font-medium transition",
                        preferredSite === preset.id
                          ? "border-accent-500 bg-accent-100 text-accent-600"
                          : "border-border bg-surface text-text-secondary hover:border-border-strong",
                      )}
                    >
                      {preset.label}
                    </button>
                  ))}
                </div>

                {activePreset.id === "custom" && (
                  <input
                    value={customOrigin}
                    onChange={(e) => setCustomOrigin(e.target.value)}
                    placeholder="https://your-imageboard.example"
                    spellCheck={false}
                    className="mt-3 w-full rounded-lg border border-border bg-surface px-3 py-2 text-xs text-text-primary placeholder:text-text-tertiary outline-none focus:border-accent-500"
                  />
                )}

                {activeOrigin && (
                  <button
                    onClick={() => void openUrl(activeOrigin)}
                    className="mt-3 flex items-center gap-2 text-xs font-medium text-accent-500 hover:text-accent-600"
                  >
                    <ExternalLink className="h-3.5 w-3.5" />
                    Open {activeOrigin.replace(/^https?:\/\//, "")}
                  </button>
                )}
              </section>

              <section className="mb-8">
                <h3 className="mb-1 text-xs font-medium uppercase tracking-wide text-text-tertiary">
                  Downloads
                </h3>
                <p className="mb-3 text-xs text-text-secondary">
                  Every thread is saved to{" "}
                  <code className="rounded bg-surface-hover px-1 py-0.5 text-[11px] text-text-primary">
                    /board/timestamp_board_threadId/
                  </code>{" "}
                  inside this folder.
                </p>
                <button
                  onClick={() => void pickDownloadRoot()}
                  className="flex w-full items-center gap-2 rounded-lg border border-border bg-surface px-3 py-2 text-xs text-text-primary hover:border-border-strong"
                >
                  <FolderOpen className="h-3.5 w-3.5 shrink-0 text-text-tertiary" />
                  <span className="truncate">{downloadRoot ?? "Choose a folder…"}</span>
                </button>
              </section>

              <section>
                <h3 className="mb-1 text-xs font-medium uppercase tracking-wide text-text-tertiary">
                  Concurrency
                </h3>
                <p className="mb-3 text-xs text-text-secondary">
                  How many files to download at the same time.
                </p>
                <div className="flex flex-wrap gap-2">
                  {CONCURRENCY_OPTIONS.map((n) => (
                    <button
                      key={n}
                      onClick={() => setConcurrency(n)}
                      className={clsx(
                        "h-8 w-10 rounded-lg border text-xs font-medium transition",
                        concurrency === n
                          ? "border-accent-500 bg-accent-100 text-accent-600"
                          : "border-border bg-surface text-text-secondary hover:border-border-strong",
                      )}
                    >
                      {n}
                    </button>
                  ))}
                </div>
              </section>
            </div>
          </motion.div>
        </>
      )}
    </AnimatePresence>
  );
}
