import { ChevronLeft } from 'lucide-react'
import { useNavStore } from '../../store/useNavStore'
import { useSitesStore } from '../../store/useSitesStore'
import { SiteAvatar } from '../sites/SiteAvatar'
import { WindowControls } from './WindowControls'
import { isMac } from '../../lib/platform'
import { cn } from '../../lib/cn'

export function TitleBar() {
  const view = useNavStore((s) => s.view)
  const siteId = useNavStore((s) => s.siteId)
  const boardCode = useNavStore((s) => s.boardCode)
  const threadId = useNavStore((s) => s.threadId)
  const back = useNavStore((s) => s.back)
  const canGoBack = useNavStore((s) => s.history.length > 0)
  const site = useSitesStore((s) => s.getSite(siteId))
  const mac = isMac()

  const crumbs: string[] = []
  if (view === 'bookmarks') crumbs.push('Bookmarks')
  else if (view === 'downloads') crumbs.push('Downloads')
  else if (view === 'settings') crumbs.push('Settings')
  else {
    if (site) crumbs.push(site.name)
    if (boardCode) crumbs.push(`/${boardCode}/`)
    if (view === 'thread' && threadId) crumbs.push(`Thread #${threadId}`)
  }

  return (
    <header
      data-tauri-drag-region
      className={cn('relative flex h-12 shrink-0 items-center border-b border-border-soft', mac ? 'pl-3' : 'pl-2')}
      style={{ boxShadow: '0 1px 0 0 var(--color-border-soft), var(--shadow-sm)' }}
    >
      <div className="glass pointer-events-none absolute inset-0 -z-10" />
      {mac && <WindowControls />}

      <div className={cn('flex min-w-0 flex-1 items-center gap-2.5', mac ? 'pl-3' : 'pl-1.5')}>
        {site && <SiteAvatar site={site} size={22} />}
        <button
          type="button"
          onClick={back}
          disabled={!canGoBack}
          className={cn(
            'flex shrink-0 items-center justify-center rounded-lg p-1.5 text-ink-dim transition-colors',
            canGoBack ? 'hover:bg-surface-3 hover:text-ink' : 'opacity-25',
          )}
        >
          <ChevronLeft size={16} />
        </button>
        <div className="flex min-w-0 items-center gap-1.5 text-sm text-ink-dim">
          {crumbs.map((c, i) => (
            <span key={i} className="flex items-center gap-1.5 truncate">
              {i > 0 && <span className="text-ink-faint">/</span>}
              <span className={cn('truncate', i === crumbs.length - 1 && 'font-semibold text-ink')}>{c}</span>
            </span>
          ))}
        </div>
      </div>

      {!mac && <WindowControls />}
    </header>
  )
}
