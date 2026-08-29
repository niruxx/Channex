import { create } from "zustand";
import * as api from "../lib/api";
import { dedupeFilenames } from "../lib/format";
import { computeThreadDownloadPath } from "../lib/path";
import { getStoredTheme, storeTheme, watchSystemTheme, type ThemePref } from "../lib/theme";
import type {
  AppSettings,
  DownloadEntry,
  MediaFilter,
  MediaItem,
  PreferredSiteId,
  ThreadInfo,
} from "../types";

interface AppState {
  url: string;
  thread: ThreadInfo | null;
  loading: boolean;
  error: string | null;

  selected: Set<string>;
  filter: MediaFilter;
  search: string;

  settingsOpen: boolean;
  settingsLoaded: boolean;
  preferredSite: PreferredSiteId;
  customOrigin: string;
  downloadRoot: string | null;
  concurrency: number;

  downloads: Record<string, DownloadEntry>;
  isDownloading: boolean;
  downloadError: string | null;
  lastDownloadPath: string | null;
  listenersReady: boolean;

  previewId: string | null;

  theme: ThemePref;
  setTheme: (theme: ThemePref) => void;

  setUrl: (url: string) => void;
  fetchThread: () => Promise<void>;
  clearThread: () => void;
  toggleSelect: (id: string) => void;
  selectAll: () => void;
  deselectAll: () => void;
  setFilter: (f: MediaFilter) => void;
  setSearch: (s: string) => void;

  loadSettings: () => Promise<void>;
  openSettings: () => void;
  closeSettings: () => void;
  setPreferredSite: (site: PreferredSiteId) => void;
  setCustomOrigin: (origin: string) => void;
  pickDownloadRoot: () => Promise<void>;
  setConcurrency: (n: number) => void;

  ensureListeners: () => void;
  startDownload: (mode?: "selected" | "all") => Promise<void>;
  downloadSingle: (id: string) => Promise<void>;
  cancelDownload: () => Promise<void>;

  openPreview: (id: string) => void;
  closePreview: () => void;
  previewNext: () => void;
  previewPrev: () => void;
}

function visibleMedia(thread: ThreadInfo | null, filter: MediaFilter, search: string): MediaItem[] {
  if (!thread) return [];
  const q = search.trim().toLowerCase();
  return thread.media.filter((m) => {
    if (filter === "images" && m.isVideo) return false;
    if (filter === "videos" && !m.isVideo) return false;
    if (q && !m.originalName.toLowerCase().includes(q)) return false;
    return true;
  });
}

function currentSettings(s: AppState): AppSettings {
  return {
    preferredSite: s.preferredSite,
    customOrigin: s.customOrigin || null,
    downloadRoot: s.downloadRoot,
    concurrency: s.concurrency,
  };
}

function persistSettings(s: AppState) {
  void api.saveSettings(currentSettings(s));
}

export const useAppStore = create<AppState>((set, get) => ({
  url: "",
  thread: null,
  loading: false,
  error: null,

  selected: new Set(),
  filter: "all",
  search: "",

  settingsOpen: false,
  settingsLoaded: false,
  preferredSite: "4chan",
  customOrigin: "",
  downloadRoot: null,
  concurrency: 6,

  downloads: {},
  isDownloading: false,
  downloadError: null,
  lastDownloadPath: null,
  listenersReady: false,

  previewId: null,

  theme: getStoredTheme(),
  setTheme: (theme) => {
    storeTheme(theme);
    set({ theme });
  },

  setUrl: (url) => set({ url }),

  fetchThread: async () => {
    const { url } = get();
    if (!url.trim()) return;
    set({ loading: true, error: null });
    try {
      const thread = await api.fetchThreadInfo(url.trim());
      set({
        thread,
        loading: false,
        selected: new Set(thread.media.map((m) => m.id)),
        downloads: {},
        downloadError: null,
        lastDownloadPath: null,
        previewId: null,
      });
    } catch (err) {
      set({ loading: false, error: String(err) });
    }
  },

  clearThread: () => {
    if (get().isDownloading) {
      void api.cancelDownloads();
    }
    set({
      url: "",
      thread: null,
      loading: false,
      error: null,
      selected: new Set(),
      filter: "all",
      search: "",
      downloads: {},
      isDownloading: false,
      downloadError: null,
      lastDownloadPath: null,
      previewId: null,
    });
  },

  toggleSelect: (id) =>
    set((s) => {
      const next = new Set(s.selected);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return { selected: next };
    }),

  selectAll: () =>
    set((s) => ({
      selected: new Set(visibleMedia(s.thread, s.filter, s.search).map((m) => m.id)),
    })),

  deselectAll: () => set({ selected: new Set() }),

  setFilter: (filter) => set({ filter }),
  setSearch: (search) => set({ search }),

  loadSettings: async () => {
    if (get().settingsLoaded) return;
    const settings = await api.loadSettings();
    let downloadRoot = settings.downloadRoot;
    if (!downloadRoot) {
      downloadRoot = (await api.defaultDownloadDir()) ?? null;
    }
    set({
      settingsLoaded: true,
      preferredSite: settings.preferredSite,
      customOrigin: settings.customOrigin ?? "",
      downloadRoot,
      concurrency: settings.concurrency,
    });
    if (downloadRoot !== settings.downloadRoot) {
      persistSettings(get());
    }
  },

  openSettings: () => set({ settingsOpen: true }),
  closeSettings: () => set({ settingsOpen: false }),

  setPreferredSite: (site) => {
    set({ preferredSite: site });
    persistSettings(get());
  },

  setCustomOrigin: (origin) => {
    set({ customOrigin: origin });
    persistSettings(get());
  },

  pickDownloadRoot: async () => {
    const dir = await api.pickDirectory();
    if (dir) {
      set({ downloadRoot: dir });
      persistSettings(get());
    }
  },

  setConcurrency: (n) => {
    set({ concurrency: n });
    persistSettings(get());
  },

  ensureListeners: () => {
    if (get().listenersReady) return;
    set({ listenersReady: true });

    watchSystemTheme(() => get().theme);

    api.onDownloadProgress((p) => {
      set((s) => ({
        downloads: {
          ...s.downloads,
          [p.id]: {
            status: "downloading",
            downloaded: p.downloaded,
            total: p.total,
          },
        },
      }));
    });

    api.onDownloadDone((p) => {
      set((s) => {
        const prev = s.downloads[p.id];
        const next = {
          ...s.downloads,
          [p.id]: {
            status: "done" as const,
            downloaded: prev?.total ?? prev?.downloaded ?? 0,
            total: prev?.total ?? null,
          },
        };
        const stillDownloading = Object.values(next).some(
          (d) => d.status === "downloading" || d.status === "queued",
        );
        return { downloads: next, isDownloading: stillDownloading };
      });
    });

    api.onDownloadError((p) => {
      set((s) => {
        const next = {
          ...s.downloads,
          [p.id]: {
            status: p.message === "Cancelled" ? "cancelled" : "error",
            downloaded: s.downloads[p.id]?.downloaded ?? 0,
            total: s.downloads[p.id]?.total ?? null,
            error: p.message,
          } as DownloadEntry,
        };
        const stillDownloading = Object.values(next).some(
          (d) => d.status === "downloading" || d.status === "queued",
        );
        return { downloads: next, isDownloading: stillDownloading };
      });
    });
  },

  startDownload: async (mode = "selected") => {
    const { thread, selected, downloadRoot, concurrency } = get();
    if (!thread || !downloadRoot) return;

    const items = mode === "all" ? thread.media : thread.media.filter((m) => selected.has(m.id));
    if (items.length === 0) return;
    const filenames = dedupeFilenames(items);
    const targetDir = computeThreadDownloadPath(downloadRoot, thread.board, thread.threadNo);

    const initialDownloads: Record<string, DownloadEntry> = {};
    for (const item of items) {
      initialDownloads[item.id] = { status: "queued", downloaded: 0, total: item.sizeBytes };
    }
    set({ downloads: initialDownloads, isDownloading: true, downloadError: null, lastDownloadPath: targetDir });

    const requestItems = items.map((item) => ({
      id: item.id,
      url: item.mediaUrl,
      filename: filenames.get(item) ?? item.originalName,
    }));

    try {
      await api.downloadFiles(requestItems, targetDir, concurrency);
    } catch (err) {
      set({ downloadError: String(err) });
    } finally {
      set({ isDownloading: false });
    }
  },

  downloadSingle: async (id) => {
    const { thread, downloadRoot } = get();
    if (!thread || !downloadRoot) return;
    const item = thread.media.find((m) => m.id === id);
    if (!item) return;

    const targetDir = computeThreadDownloadPath(downloadRoot, thread.board, thread.threadNo);
    set((s) => ({
      downloads: {
        ...s.downloads,
        [id]: { status: "queued", downloaded: 0, total: item.sizeBytes },
      },
      isDownloading: true,
      downloadError: null,
      lastDownloadPath: targetDir,
    }));

    try {
      await api.downloadFiles(
        [{ id: item.id, url: item.mediaUrl, filename: item.originalName }],
        targetDir,
        1,
      );
    } catch (err) {
      set({ downloadError: String(err) });
    } finally {
      set({ isDownloading: false });
    }
  },

  cancelDownload: async () => {
    await api.cancelDownloads();
  },

  openPreview: (id) => set({ previewId: id }),
  closePreview: () => set({ previewId: null }),

  previewNext: () =>
    set((s) => {
      if (!s.previewId) return {};
      const list = visibleMedia(s.thread, s.filter, s.search);
      const idx = list.findIndex((m) => m.id === s.previewId);
      if (idx === -1 || list.length === 0) return {};
      return { previewId: list[(idx + 1) % list.length].id };
    }),

  previewPrev: () =>
    set((s) => {
      if (!s.previewId) return {};
      const list = visibleMedia(s.thread, s.filter, s.search);
      const idx = list.findIndex((m) => m.id === s.previewId);
      if (idx === -1 || list.length === 0) return {};
      return { previewId: list[(idx - 1 + list.length) % list.length].id };
    }),
}));

export function selectVisibleMedia(state: AppState): MediaItem[] {
  return visibleMedia(state.thread, state.filter, state.search);
}
