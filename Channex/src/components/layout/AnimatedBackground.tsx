import { useMemo } from 'react'
import { useSettingsStore } from '../../store/useSettingsStore'

export function AnimatedBackground() {
  const theme = useSettingsStore((s) => s.backgroundTheme)

  if (theme === 'aurora') return <AuroraLayer />
  if (theme === 'particles') return <ParticlesLayer />
  if (theme === 'grid') return <div className="bg-layer grid-layer" />
  return null
}

function AuroraLayer() {
  return (
    <div className="bg-layer">
      <div className="aurora-blob aurora-blob-1" />
      <div className="aurora-blob aurora-blob-2" />
      <div className="aurora-blob aurora-blob-3" />
    </div>
  )
}

function ParticlesLayer() {
  const particles = useMemo(
    () =>
      Array.from({ length: 26 }, (_, i) => ({
        id: i,
        left: Math.random() * 100,
        size: 2 + Math.random() * 4,
        duration: 14 + Math.random() * 16,
        delay: -Math.random() * 24,
        drift: (Math.random() - 0.5) * 120,
      })),
    [],
  )

  return (
    <div className="bg-layer">
      {particles.map((p) => (
        <span
          key={p.id}
          className="particle"
          style={{
            left: `${p.left}%`,
            width: p.size,
            height: p.size,
            animationDuration: `${p.duration}s`,
            animationDelay: `${p.delay}s`,
            ['--drift' as string]: `${p.drift}px`,
          }}
        />
      ))}
    </div>
  )
}
