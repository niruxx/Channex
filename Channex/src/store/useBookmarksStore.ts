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
  getBookmark: (siteId: string, boardCode: string, threadId: string) => Bookmark | undefined
  add: (bookmark: Bookmark) => Promise<void>
  remove: (siteId: string, boardCode: string, threadId: string) => Promise<void>
  toggle: (bookmark: Bookmark) => Promise<void>
  markSeen: (siteId: string, boardCode: string, threadId: string, replyCount: number) => Promise<void>
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

  getBookmark: (siteId, boardCode, threadId) =>
    get().bookmarks.find((b) => key(b.siteId, b.boardCode, b.threadId) === key(siteId, boardCode, threadId)),

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

  // Called when a bookmarked thread is actually read, so the "new replies since
  // you last looked" badge in the catalog clears itself.
  markSeen: async (siteId, boardCode, threadId, replyCount) => {
    const k = key(siteId, boardCode, threadId)
    const current = get().bookmarks.find((b) => key(b.siteId, b.boardCode, b.threadId) === k)
    if (!current || current.lastSeenReplyCount === replyCount) return
    const next = get().bookmarks.map((b) => (key(b.siteId, b.boardCode, b.threadId) === k ? { ...b, lastSeenReplyCount: replyCount } : b))
    set({ bookmarks: next })
    const store = await getStore()
    await store.set('bookmarks', next)
  },
}))
