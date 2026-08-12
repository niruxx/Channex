import { create } from 'zustand'
import type { Bookmark } from '../types'
import { getStore } from '../lib/persist'

function key(siteId: string, boardCode: string, threadId: string) {
  return `${siteId}/${boardCode}/${threadId}`
}

interface BookmarksState {
  bookmarks: Bookmark[]
  hydrated: boolean
  hydrate: () => Promise<void>
  isBookmarked: (siteId: string, boardCode: string, threadId: string) => boolean
  add: (bookmark: Bookmark) => Promise<void>
  remove: (siteId: string, boardCode: string, threadId: string) => Promise<void>
  toggle: (bookmark: Bookmark) => Promise<void>
}

export const useBookmarksStore = create<BookmarksState>((set, get) => ({
  bookmarks: [],
  hydrated: false,

  hydrate: async () => {
    if (get().hydrated) return
    const store = await getStore()
    const bookmarks = (await store.get<Bookmark[]>('bookmarks')) ?? []
    set({ bookmarks, hydrated: true })
  },

  isBookmarked: (siteId, boardCode, threadId) =>
    get().bookmarks.some((b) => key(b.siteId, b.boardCode, b.threadId) === key(siteId, boardCode, threadId)),

  add: async (bookmark) => {
    const next = [bookmark, ...get().bookmarks.filter((b) => key(b.siteId, b.boardCode, b.threadId) !== key(bookmark.siteId, bookmark.boardCode, bookmark.threadId))]
    set({ bookmarks: next })
    const store = await getStore()
    await store.set('bookmarks', next)
  },

  remove: async (siteId, boardCode, threadId) => {
    const next = get().bookmarks.filter((b) => key(b.siteId, b.boardCode, b.threadId) !== key(siteId, boardCode, threadId))
    set({ bookmarks: next })
    const store = await getStore()
    await store.set('bookmarks', next)
  },

  toggle: async (bookmark) => {
    if (get().isBookmarked(bookmark.siteId, bookmark.boardCode, bookmark.threadId)) {
      await get().remove(bookmark.siteId, bookmark.boardCode, bookmark.threadId)
    } else {
      await get().add(bookmark)
    }
  },
}))
