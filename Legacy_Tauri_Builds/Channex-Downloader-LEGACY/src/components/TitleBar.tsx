import { useEffect, useState } from "react";
import clsx from "clsx";
import { motion } from "framer-motion";
import { Copy, GalleryThumbnails, Minus, Settings, Square, X } from "lucide-react";
import { appWindow, closeWindow, minimizeWindow, toggleMaximizeWindow } from "../lib/appWindow";
import { useAppStore } from "../store/useAppStore";

function WinButton({
  onClick,
  danger,
  children,
  label,
}: {
  onClick: () => void;
  danger?: boolean;
  children: React.ReactNode;
  label: string;
}) {
  return (
    <motion.button
      onClick={onClick}
      aria-label={label}
      whileHover={{ backgroundColor: danger ? "#d93025" : "rgba(127,127,127,0.16)" }}
      whileTap={{ scale: 0.92 }}
      className={clsx(
        "flex h-full w-11 items-center justify-center transition-colors",
        danger ? "text-text-secondary hover:text-white" : "text-text-secondary hover:text-text-primary",
      )}
    >
      {children}
    </motion.button>
  );
}

export function TitleBar() {
  const [maximized, setMaximized] = useState(false);
  const openSettings = useAppStore((s) => s.openSettings);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    appWindow.isMaximized().then(setMaximized);
    appWindow.onResized(() => {
      void appWindow.isMaximized().then(setMaximized);
    }).then((fn) => {
      unlisten = fn;
    });
    return () => unlisten?.();
  }, []);

  return (
    <div
      data-tauri-drag-region
      className="flex h-9 shrink-0 select-none items-center justify-between border-b border-border bg-surface-alt"
    >
      <div data-tauri-drag-region className="flex h-full flex-1 items-center gap-2 px-3">
        <div className="flex h-5 w-5 items-center justify-center rounded-full bg-accent-100 text-accent-500">
          <GalleryThumbnails className="h-3 w-3" />
        </div>
        <span className="text-xs font-medium tracking-wide text-text-secondary">Channex Downloader</span>
      </div>

      <div className="flex h-full items-center">
        <motion.button
          onClick={openSettings}
          aria-label="Settings"
          whileHover={{ backgroundColor: "rgba(127,127,127,0.16)" }}
          whileTap={{ scale: 0.92, rotate: 45 }}
          className="flex h-full w-11 items-center justify-center text-text-secondary transition-colors hover:text-text-primary"
        >
          <Settings className="h-3.5 w-3.5" />
        </motion.button>
        <WinButton onClick={() => void minimizeWindow()} label="Minimize">
          <Minus className="h-3.5 w-3.5" />
        </WinButton>
        <WinButton onClick={() => void toggleMaximizeWindow()} label="Maximize">
          {maximized ? <Copy className="h-3 w-3 -scale-x-100" /> : <Square className="h-3 w-3" />}
        </WinButton>
        <WinButton onClick={() => void closeWindow()} danger label="Close">
          <X className="h-3.5 w-3.5" />
        </WinButton>
      </div>
    </div>
  );
}
