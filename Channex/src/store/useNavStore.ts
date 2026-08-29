import { create } from 'zustand'

export type ViewMode = 'catalog' | 'thread' | 'bookmarks' | 'downloads' | 'settings'

interface NavFrame {
  view: ViewMode
  siteId: string
  boardCode: string | null
  threadId: string | null
}

interface NavState extends NavFrame {
  history: NavFrame[]
  setSite: (siteId: string) => void
  goCatalog: (siteId: string, boardCode: string) => void
  goThread: (siteId: string, boardCode: string, threadId: string) => void
  goBookmarks: () => void
  goDownloads: () => void
  goSettings: () => void
  back: () => void
  canGoBack: () => boolean
}

const initial: NavFrame = { view: 'catalog', siteId: '4chan', boardCode: 'g', threadId: null }

export const useNavStore = create<NavState>((set, get) => ({
  ...initial,
  history: [],

  setSite: (siteId) =>
    set((s) => ({
      history: [...s.history, frameOf(s)],
      siteId,
      view: 'catalog',
      boardCode: null,
      threadId: null,
    })),

  goCatalog: (siteId, boardCode) =>
    set((s) => ({
      history: [...s.history, frameOf(s)],
      view: 'catalog',
      siteId,
      boardCode,
      threadId: null,
    })),

  goThread: (siteId, boardCode, threadId) =>
    set((s) => ({
      history: [...s.history, frameOf(s)],
      view: 'thread',
      siteId,
      boardCode,
      threadId,
    })),

  goBookmarks: () => set((s) => ({ history: [...s.history, frameOf(s)], view: 'bookmarks' })),
  goDownloads: () => set((s) => ({ history: [...s.history, frameOf(s)], view: 'downloads' })),
  goSettings: () => set((s) => ({ history: [...s.history, frameOf(s)], view: 'settings' })),

  back: () =>
    set((s) => {
      const h = [...s.history]
      const prev = h.pop()
      if (!prev) return s
      return { ...prev, history: h }
    }),

  canGoBack: () => get().history.length > 0,
}))

function frameOf(s: NavState): NavFrame {
  return { view: s.view, siteId: s.siteId, boardCode: s.boardCode, threadId: s.threadId }
}
