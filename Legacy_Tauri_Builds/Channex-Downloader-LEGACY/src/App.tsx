import { useEffect, useState } from "react";
import { motion } from "framer-motion";
import { TitleBar } from "./components/TitleBar";
import { UrlBar } from "./components/UrlBar";
import { ThreadHeader } from "./components/ThreadHeader";
import { Toolbar } from "./components/Toolbar";
import { MediaGrid } from "./components/MediaGrid";
import { DownloadBar } from "./components/DownloadBar";
import { EmptyState } from "./components/EmptyState";
import { SettingsPanel } from "./components/SettingsPanel";
import { PreviewModal } from "./components/PreviewModal";
import { ResizeHandles } from "./components/ResizeHandles";
import { appWindow, destroyWindow } from "./lib/appWindow";
import { useAppStore } from "./store/useAppStore";

export default function App() {
  const thread = useAppStore((s) => s.thread);
  const ensureListeners = useAppStore((s) => s.ensureListeners);
  const loadSettings = useAppStore((s) => s.loadSettings);
  const [closing, setClosing] = useState(false);

  useEffect(() => {
    ensureListeners();
    void loadSettings();
  }, [ensureListeners, loadSettings]);

  useEffect(() => {
    let active = true;
    let unlisten: (() => void) | undefined;
    appWindow
      .onCloseRequested((event) => {
        event.preventDefault();
        setClosing(true);
      })
      .then((fn) => {
        if (active) unlisten = fn;
        else fn();
      });
    return () => {
      active = false;
      unlisten?.();
    };
  }, []);

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: closing ? 0 : 1 }}
      transition={{ duration: 0.18, ease: "easeInOut" }}
      onAnimationComplete={() => {
        if (closing) void destroyWindow();
      }}
      className="flex h-screen flex-col bg-surface"
    >
      <TitleBar />

      <header className="flex items-center gap-4 border-b border-border bg-surface-alt px-6 py-4">
        <div className="flex-1">
          <UrlBar />
        </div>
      </header>

      <main className="flex flex-1 flex-col gap-3 overflow-y-auto px-6 py-4">
        {thread && (
          <>
            <ThreadHeader />
            <Toolbar />
          </>
        )}
        {thread ? <MediaGrid /> : <EmptyState />}
      </main>

      <DownloadBar />
      <SettingsPanel />
      <PreviewModal />
      <ResizeHandles />
    </motion.div>
  );
}
