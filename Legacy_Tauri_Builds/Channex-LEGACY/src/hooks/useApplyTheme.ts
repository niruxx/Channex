import { useEffect } from 'react'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useSettingsStore } from '../store/useSettingsStore'

function resolveTheme(mode: string, systemPrefersDark: boolean): 'dark' | 'light' {
  if (mode === 'system') return systemPrefersDark ? 'dark' : 'light'
  return mode === 'light' ? 'light' : 'dark'
}

export function useApplyTheme() {
  const theme = useSettingsStore((s) => s.theme)
  const accentColor = useSettingsStore((s) => s.accentColor)
  const hydrated = useSettingsStore((s) => s.hydrated)

  useEffect(() => {
    if (!hydrated) return
    const media = window.matchMedia('(prefers-color-scheme: dark)')

    function apply() {
      const resolved = resolveTheme(theme, media.matches)
      document.documentElement.dataset.theme = resolved
      getCurrentWindow()
        .setTheme(resolved)
        .catch(() => {})
    }

    apply()
    if (theme === 'system') {
      media.addEventListener('change', apply)
      return () => media.removeEventListener('change', apply)
    }
  }, [theme, hydrated])

  useEffect(() => {
    if (!hydrated) return
    document.documentElement.style.setProperty('--color-accent', accentColor)
  }, [accentColor, hydrated])
}
