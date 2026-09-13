import { useEffect } from 'react'
import { useNavStore } from '../store/useNavStore'

function isTypingTarget(el: EventTarget | null): boolean {
  if (!(el instanceof HTMLElement)) return false
  return el.isContentEditable || el.tagName === 'INPUT' || el.tagName === 'TEXTAREA' || el.tagName === 'SELECT'
}

/**
 * App-wide shortcuts. Views that can refresh listen for the `imageboarder:refresh`
 * event rather than being wired through here.
 */
export function useKeyboardShortcuts() {
  useEffect(() => {
    function onKey(e: KeyboardEvent) {
      // Never steal keys from the lightbox (it owns Esc / arrows while open).
      if (document.querySelector('[data-lightbox-open]')) return

      const typing = isTypingTarget(e.target)

      if (e.key === 'Escape' && !typing) {
        useNavStore.getState().back()
        return
      }

      if (typing || e.ctrlKey || e.metaKey || e.altKey) return

      if (e.key === '/') {
        const search = document.querySelector<HTMLInputElement>('[data-catalog-search]')
        if (search) {
          e.preventDefault()
          search.focus()
          search.select()
        }
        return
      }

      switch (e.key.toLowerCase()) {
        case 'r':
          window.dispatchEvent(new CustomEvent('imageboarder:refresh'))
          break
        case 'b':
          useNavStore.getState().goBookmarks()
          break
        case 'd':
          useNavStore.getState().goDownloads()
          break
        case ',':
          useNavStore.getState().goSettings()
          break
      }
    }

    window.addEventListener('keydown', onKey)
    return () => window.removeEventListener('keydown', onKey)
  }, [])
}
