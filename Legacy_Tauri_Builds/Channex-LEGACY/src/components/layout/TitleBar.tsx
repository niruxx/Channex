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

  const goCatalog = useNavStore((s) => s.goCatalog)

  type Crumb = { label: string; onClick?: () => void }
  const crumbs: Crumb[] = []
  if (view === 'bookmarks') crumbs.push({ label: 'Bookmarks' })
  else if (view === 'downloads') crumbs.push({ label: 'Downloads' })
  else if (view === 'settings') crumbs.push({ label: 'Settings' })
  else {
    if (site) crumbs.push({ label: site.name })
    if (boardCode) {
      crumbs.push({
        label: `/${boardCode}/`,
        // Only a link when it actually goes somewhere — on the catalog itself it's the current page.
        onClick: view === 'thread' ? () => goCatalog(siteId, boardCode) : undefined,
      })
    }
    if (view === 'thread' && threadId) crumbs.push({ label: `Thread #${threadId}` })
  }

  return (
    <header
      data-tauri-drag-region
      className={cn('relative flex h-14 shrink-0 items-center border-b border-border-soft', mac ? 'pl-4' : 'pl-2')}
      style={{ boxShadow: '0 1px 0 0 var(--color-border-soft), var(--shadow-sm)' }}
    >
      <div className="glass pointer-events-none absolute inset-0 -z-10" />
      {mac && <WindowControls />}

      <div className={cn('flex min-w-0 flex-1 items-center gap-3', mac ? 'pl-3' : 'pl-2')}>
        {site && <SiteAvatar site={site} size={24} />}
        <button
          type="button"
          onClick={back}
          disabled={!canGoBack}
          className={cn(
            'flex shrink-0 items-center justify-center rounded-lg p-2 text-ink-dim transition-colors',
            canGoBack ? 'hover:bg-surface-3 hover:text-ink' : 'opacity-25',
          )}
        >
          <ChevronLeft size={18} />
        </button>
        <div className="flex min-w-0 items-center gap-2 text-sm text-ink-dim">
          {crumbs.map((c, i) => (
            <span key={i} className="flex items-center gap-2 truncate">
              {i > 0 && <span className="text-ink-faint">/</span>}
              {c.onClick ? (
                <button
                  type="button"
                  onClick={c.onClick}
                  className="truncate rounded px-1.5 py-1 transition-colors hover:bg-surface-3 hover:text-ink"
                >
                  {c.label}
                </button>
              ) : (
                <span className={cn('truncate', i === crumbs.length - 1 && 'font-semibold text-ink')}>{c.label}</span>
              )}
            </span>
          ))}
        </div>
      </div>

      {!mac && <WindowControls />}
    </header>
  )
}
