import { Bookmark, Download, Settings } from 'lucide-react'
import { SiteSwitcher } from '../sites/SiteSwitcher'
import { BoardList } from '../sites/BoardList'
import { useNavStore } from '../../store/useNavStore'
import { useDownloadsStore } from '../../store/useDownloadsStore'
import { useBookmarksStore } from '../../store/useBookmarksStore'
import { cn } from '../../lib/cn'

export function Sidebar() {
  const view = useNavStore((s) => s.view)
  const goBookmarks = useNavStore((s) => s.goBookmarks)
  const goDownloads = useNavStore((s) => s.goDownloads)
  const goSettings = useNavStore((s) => s.goSettings)
  const activeDownloads = useDownloadsStore((s) =>
    s.jobs.reduce((n, j) => n + j.items.filter((i) => i.status === 'pending' || i.status === 'downloading').length, 0),
  )
  const bookmarkCount = useBookmarksStore((s) => s.bookmarks.length)

  return (
    <aside className="glass flex h-full w-72 shrink-0 flex-col border-r border-border-soft">
      <div className="px-4 pb-1 pt-4">
        <p className="section-label mb-3">Sites</p>
        <SiteSwitcher />
      </div>

      <div className="mx-4 my-2 h-px bg-border-soft" />

      <div className="flex min-h-0 flex-1 flex-col px-4 pb-2">
        <p className="section-label mb-2.5 shrink-0">Boards</p>
        <BoardList />
      </div>

      <nav className="flex flex-col gap-1 border-t border-border-soft p-3">
        <NavButton icon={Bookmark} label="Bookmarks" active={view === 'bookmarks'} onClick={goBookmarks} badge={bookmarkCount || undefined} muted />
        <NavButton icon={Download} label="Downloads" active={view === 'downloads'} onClick={goDownloads} badge={activeDownloads || undefined} />
        <NavButton icon={Settings} label="Settings" active={view === 'settings'} onClick={goSettings} />
      </nav>
    </aside>
  )
}

function NavButton({
  icon: Icon,
  label,
  active,
  onClick,
  badge,
  muted,
}: {
  icon: typeof Bookmark
  label: string
  active: boolean
  onClick: () => void
  badge?: number
  /** Informational counts stay quiet; only live activity earns the accent badge. */
  muted?: boolean
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={cn(
        'group relative flex items-center gap-3 overflow-hidden rounded-xl px-3 py-2.5 text-sm transition-colors',
        active ? 'bg-surface-3 text-ink' : 'text-ink-dim hover:bg-surface-2 hover:text-ink',
      )}
    >
      <span
        className={cn(
          'absolute inset-y-1.5 left-0 w-[3px] rounded-full bg-accent transition-transform duration-200',
          active ? 'scale-y-100' : 'scale-y-0',
        )}
      />
      <Icon size={17} strokeWidth={2} className={cn('transition-colors', active && 'text-accent')} />
      <span className="flex-1 text-left font-medium">{label}</span>
      {!!badge && (
        <span
          className={cn(
            'rounded-full px-1.5 py-0.5 text-[10px] font-semibold',
            muted ? 'bg-surface-3 text-ink-faint' : 'bg-accent text-accent-ink',
          )}
        >
          {badge}
        </span>
      )}
    </button>
  )
}
