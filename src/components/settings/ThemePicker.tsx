import { Monitor, Moon, Sun } from 'lucide-react'
import { useSettingsStore, type ThemeMode } from '../../store/useSettingsStore'
import { cn } from '../../lib/cn'

const THEME_OPTIONS: { mode: ThemeMode; label: string; icon: typeof Sun }[] = [
  { mode: 'light', label: 'Light', icon: Sun },
  { mode: 'dark', label: 'Dark', icon: Moon },
  { mode: 'system', label: 'System', icon: Monitor },
]

export function ThemePicker() {
  const theme = useSettingsStore((s) => s.theme)
  const setTheme = useSettingsStore((s) => s.setTheme)

  return (
    <div className="grid grid-cols-3 gap-2">
      {THEME_OPTIONS.map(({ mode, label, icon: Icon }) => {
        const active = theme === mode
        return (
          <button
            key={mode}
            type="button"
            onClick={() => setTheme(mode)}
            className={cn(
              'flex flex-col items-center gap-2 rounded-xl border px-3 py-3 text-xs font-medium transition-colors',
              active ? 'border-accent/60 bg-accent/10 text-ink' : 'border-border-soft bg-surface-3 text-ink-dim hover:text-ink',
            )}
          >
            <Icon size={17} className={active ? 'text-accent' : undefined} />
            {label}
          </button>
        )
      })}
    </div>
  )
}
