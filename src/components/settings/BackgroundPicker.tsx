import { Ban, Grid3x3, Sparkles, Wind } from 'lucide-react'
import { BACKGROUND_THEMES, useSettingsStore, type BackgroundTheme } from '../../store/useSettingsStore'
import { cn } from '../../lib/cn'

const ICONS: Record<BackgroundTheme, typeof Sparkles> = {
  none: Ban,
  aurora: Sparkles,
  particles: Wind,
  grid: Grid3x3,
}

export function BackgroundPicker() {
  const backgroundTheme = useSettingsStore((s) => s.backgroundTheme)
  const setBackgroundTheme = useSettingsStore((s) => s.setBackgroundTheme)

  return (
    <div className="grid grid-cols-2 gap-2 sm:grid-cols-4">
      {BACKGROUND_THEMES.map(({ value, name, description }) => {
        const Icon = ICONS[value]
        const active = backgroundTheme === value
        return (
          <button
            key={value}
            type="button"
            title={description}
            onClick={() => setBackgroundTheme(value)}
            className={cn(
              'flex flex-col items-center gap-2 rounded-xl border px-3 py-3 text-xs font-medium transition-colors',
              active ? 'border-accent/60 bg-accent/10 text-ink' : 'border-border-soft bg-surface-3 text-ink-dim hover:text-ink',
            )}
          >
            <Icon size={17} className={active ? 'text-accent' : undefined} />
            {name}
          </button>
        )
      })}
    </div>
  )
}
