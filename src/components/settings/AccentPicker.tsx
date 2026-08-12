import { Check } from 'lucide-react'
import { motion } from 'motion/react'
import { ACCENT_PRESETS, useSettingsStore } from '../../store/useSettingsStore'

export function AccentPicker() {
  const accentColor = useSettingsStore((s) => s.accentColor)
  const setAccentColor = useSettingsStore((s) => s.setAccentColor)

  return (
    <div className="flex flex-wrap gap-2.5">
      {ACCENT_PRESETS.map((preset) => {
        const active = accentColor.toLowerCase() === preset.value.toLowerCase()
        return (
          <motion.button
            key={preset.value}
            type="button"
            title={preset.name}
            onClick={() => setAccentColor(preset.value)}
            whileTap={{ scale: 0.9 }}
            whileHover={{ scale: 1.08 }}
            className="flex size-8 items-center justify-center rounded-full"
            style={{ background: preset.value, boxShadow: active ? `0 0 0 2px var(--color-surface-2), 0 0 0 4px ${preset.value}` : 'var(--shadow-sm)' }}
          >
            {active && <Check size={14} className="text-accent-ink" strokeWidth={3} />}
          </motion.button>
        )
      })}
      <label
        title="Custom color"
        className="relative flex size-8 cursor-pointer items-center justify-center overflow-hidden rounded-full border border-dashed border-border text-ink-faint transition-colors hover:border-accent hover:text-accent"
      >
        +
        <input
          type="color"
          value={accentColor}
          onChange={(e) => setAccentColor(e.target.value)}
          className="absolute inset-0 cursor-pointer opacity-0"
        />
      </label>
    </div>
  )
}
