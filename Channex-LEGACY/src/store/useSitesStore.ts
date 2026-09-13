import { create } from 'zustand'
import type { Board, ChanSite } from '../types'
import { PRESET_SITES, makeCustomSite } from '../lib/sites'
import { getAdapter } from '../lib/adapters'
import { getStore } from '../lib/persist'
import { uid } from '../lib/format'

interface SitesState {
  sites: ChanSite[]
  boards: Record<string, Board[]>
  boardsLoading: Record<string, boolean>
  boardsError: Record<string, string | undefined>
  hydrated: boolean

  hydrate: () => Promise<void>
  getSite: (siteId: string) => ChanSite | undefined
  addCustomSite: (input: Parameters<typeof makeCustomSite>[0]) => Promise<ChanSite>
  removeCustomSite: (id: string) => Promise<void>
  loadBoards: (siteId: string, force?: boolean) => Promise<void>
  addManualBoard: (siteId: string, code: string, title?: string) => void
}

export const useSitesStore = create<SitesState>((set, get) => ({
  sites: PRESET_SITES,
  boards: {},
  boardsLoading: {},
  boardsError: {},
  hydrated: false,

  hydrate: async () => {
    if (get().hydrated) return
    const store = await getStore()
    const customSites = (await store.get<ChanSite[]>('customSites')) ?? []
    set({ sites: [...PRESET_SITES, ...customSites], hydrated: true })
  },

  getSite: (siteId) => get().sites.find((s) => s.id === siteId),

  addCustomSite: async (input) => {
    const site = makeCustomSite({ ...input, id: input.id || uid() })
    const next = [...get().sites, site]
    set({ sites: next })
    const store = await getStore()
    await store.set('customSites', next.filter((s) => s.isCustom))
    return site
  },

  removeCustomSite: async (id) => {
    const next = get().sites.filter((s) => s.id !== id)
    set({ sites: next })
    const store = await getStore()
    await store.set('customSites', next.filter((s) => s.isCustom))
  },

  loadBoards: async (siteId, force = false) => {
    const site = get().getSite(siteId)
    if (!site) return
    if (!force && (get().boards[siteId]?.length || get().boardsLoading[siteId])) return
    set((s) => ({ boardsLoading: { ...s.boardsLoading, [siteId]: true }, boardsError: { ...s.boardsError, [siteId]: undefined } }))
    try {
      const adapter = getAdapter(site)
      const fetched = await adapter.fetchBoards(site)
      const byCode = new Map<string, Board>()
      for (const b of site.defaultBoards) byCode.set(b.code, b)
      for (const b of fetched) byCode.set(b.code, b)
      const merged = Array.from(byCode.values()).sort((a, b) => a.code.localeCompare(b.code))
      set((s) => ({ boards: { ...s.boards, [siteId]: merged }, boardsLoading: { ...s.boardsLoading, [siteId]: false } }))
    } catch (e) {
      set((s) => ({
        boardsLoading: { ...s.boardsLoading, [siteId]: false },
        boardsError: { ...s.boardsError, [siteId]: e instanceof Error ? e.message : String(e) },
        boards: { ...s.boards, [siteId]: s.boards[siteId] ?? site.defaultBoards },
      }))
    }
  },

  addManualBoard: (siteId, code, title) => {
    set((s) => {
      const existing = s.boards[siteId] ?? []
      if (existing.some((b) => b.code === code)) return s
      const next = [...existing, { code, title: title || code }].sort((a, b) => a.code.localeCompare(b.code))
      return { boards: { ...s.boards, [siteId]: next } }
    })
  },
}))
