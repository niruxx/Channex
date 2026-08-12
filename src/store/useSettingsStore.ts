import { create } from 'zustand'
import { getStore } from '../lib/persist'

export type ThemeMode = 'dark' | 'light' | 'system'
export type BackgroundTheme = 'none' | 'aurora' | 'particles' | 'grid'
export type CatalogViewMode = 'grid' | 'compact' | 'list'

export const ACCENT_PRESETS: { name: string; value: string }[] = [
  { name: 'Mint', value: '#6ee7c9' },
  { name: 'Violet', value: '#8b5cf6' },
  { name: 'Sky', value: '#38bdf8' },
  { name: 'Rose', value: '#fb7185' },
  { name: 'Amber', value: '#fbbf24' },
  { name: 'Lime', value: '#a3e635' },
]

export const BACKGROUND_THEMES: { value: BackgroundTheme; name: string; description: string }[] = [
  { value: 'none', name: 'None', description: 'Plain, static background' },
  { value: 'aurora', name: 'Aurora', description: 'Soft drifting color blobs' },
  { value: 'particles', name: 'Particles', description: 'Gentle rising motes' },
  { value: 'grid', name: 'Grid', description: 'Slow-panning tech grid' },
]

interface SettingsState {
  downloadDir: string | null
  blurNsfw: boolean
  hideNsfwSites: boolean
  theme: ThemeMode
  accentColor: string
  backgroundTheme: BackgroundTheme
  catalogViewMode: CatalogViewMode
  muteWebmsByDefault: boolean
  hasCompletedOnboarding: boolean
  hydrated: boolean
  hydrate: () => Promise<void>
  setDownloadDir: (dir: string | null) => Promise<void>
  setBlurNsfw: (v: boolean) => Promise<void>
  setHideNsfwSites: (v: boolean) => Promise<void>
  setTheme: (theme: ThemeMode) => Promise<void>
  setAccentColor: (color: string) => Promise<void>
  setBackgroundTheme: (theme: BackgroundTheme) => Promise<void>
  setCatalogViewMode: (mode: CatalogViewMode) => Promise<void>
  setMuteWebmsByDefault: (v: boolean) => Promise<void>
  setHasCompletedOnboarding: (v: boolean) => Promise<void>
}

export const useSettingsStore = create<SettingsState>((set, get) => ({
  downloadDir: null,
  blurNsfw: true,
  hideNsfwSites: false,
  theme: 'dark',
  accentColor: ACCENT_PRESETS[0].value,
  backgroundTheme: 'none',
  catalogViewMode: 'grid',
  muteWebmsByDefault: true,
  hasCompletedOnboarding: false,
  hydrated: false,

  hydrate: async () => {
    if (get().hydrated) return
    const store = await getStore()
    const downloadDir = (await store.get<string>('downloadDir')) ?? null
    const blurNsfw = (await store.get<boolean>('blurNsfw')) ?? true
    const hideNsfwSites = (await store.get<boolean>('hideNsfwSites')) ?? false
    const theme = (await store.get<ThemeMode>('theme')) ?? 'dark'
    const accentColor = (await store.get<string>('accentColor')) ?? ACCENT_PRESETS[0].value
    const backgroundTheme = (await store.get<BackgroundTheme>('backgroundTheme')) ?? 'none'
    const catalogViewMode = (await store.get<CatalogViewMode>('catalogViewMode')) ?? 'grid'
    const muteWebmsByDefault = (await store.get<boolean>('muteWebmsByDefault')) ?? true
    const hasCompletedOnboarding = (await store.get<boolean>('hasCompletedOnboarding')) ?? false
    set({
      downloadDir,
      blurNsfw,
      hideNsfwSites,
      theme,
      accentColor,
      backgroundTheme,
      catalogViewMode,
      muteWebmsByDefault,
      hasCompletedOnboarding,
      hydrated: true,
    })
  },

  setDownloadDir: async (dir) => {
    set({ downloadDir: dir })
    const store = await getStore()
    await store.set('downloadDir', dir)
  },

  setBlurNsfw: async (v) => {
    set({ blurNsfw: v })
    const store = await getStore()
    await store.set('blurNsfw', v)
  },

  setHideNsfwSites: async (v) => {
    set({ hideNsfwSites: v })
    const store = await getStore()
    await store.set('hideNsfwSites', v)
  },

  setTheme: async (theme) => {
    set({ theme })
    const store = await getStore()
    await store.set('theme', theme)
  },

  setAccentColor: async (color) => {
    set({ accentColor: color })
    const store = await getStore()
    await store.set('accentColor', color)
  },

  setBackgroundTheme: async (backgroundTheme) => {
    set({ backgroundTheme })
    const store = await getStore()
    await store.set('backgroundTheme', backgroundTheme)
  },

  setCatalogViewMode: async (catalogViewMode) => {
    set({ catalogViewMode })
    const store = await getStore()
    await store.set('catalogViewMode', catalogViewMode)
  },

  setMuteWebmsByDefault: async (v) => {
    set({ muteWebmsByDefault: v })
    const store = await getStore()
    await store.set('muteWebmsByDefault', v)
  },

  setHasCompletedOnboarding: async (v) => {
    set({ hasCompletedOnboarding: v })
    const store = await getStore()
    await store.set('hasCompletedOnboarding', v)
  },
}))
