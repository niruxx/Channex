import { Grid3x3, LayoutGrid, List } from 'lucide-react'
import { useSettingsStore, type CatalogViewMode } from '../../store/useSettingsStore'
import { cn } from '../../lib/cn'

const MODES: { mode: CatalogViewMode; icon: typeof LayoutGrid; label: string }[] = [
  { mode: 'grid', icon: LayoutGrid, label: 'Grid view' },
  { mode: 'compact', icon: Grid3x3, label: 'Compact grid' },
  { mode: 'list', icon: List, label: 'List view' },
]

export function ViewModeToggle() {
  const viewMode = useSettingsStore((s) => s.catalogViewMode)
  const setViewMode = useSettingsStore((s) => s.setCatalogViewMode)

  return (
    <div className="flex items-center gap-0.5 rounded-lg bg-surface-3 p-0.5">
      {MODES.map(({ mode, icon: Icon, label }) => {
        const active = viewMode === mode
        return (
          <button
            key={mode}
            type="button"
            title={label}
            onClick={() => setViewMode(mode)}
            className={cn(
              'flex size-7 items-center justify-center rounded-md transition-colors',
              active ? 'bg-surface-4 text-accent shadow-sm' : 'text-ink-faint hover:text-ink',
            )}
          >
            <Icon size={14} />
          </button>
        )
      })}
    </div>
  )
}
