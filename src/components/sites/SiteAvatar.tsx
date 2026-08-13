import { useEffect, useState } from 'react'
import type { ChanSite } from '../../types'
import { faviconCandidates } from '../../lib/favicon'
import { cn } from '../../lib/cn'
import { BirthdayHat } from '../common/BirthdayHat'
import { useShowBirthdayHats } from '../../hooks/useShowBirthdayHats'

export function SiteAvatar({ site, size = 40, ring = false }: { site: ChanSite; size?: number; ring?: boolean }) {
  const candidates = faviconCandidates(site)
  const [index, setIndex] = useState(0)
  const showHat = useShowBirthdayHats()

  useEffect(() => {
    setIndex(0)
  }, [site.id, site.siteOrigin])

  const shadow = ring ? `0 0 0 2px var(--color-surface-2), 0 0 0 4px ${site.accent}, var(--shadow-md)` : 'var(--shadow-sm)'
  const showImage = index < candidates.length

  return (
    <span className="relative inline-flex shrink-0" style={{ width: size, height: size }}>
      <span
        className={cn(
          'flex h-full w-full items-center justify-center overflow-hidden rounded-full font-semibold text-accent-ink',
          ring && 'ring-offset-2 ring-offset-surface',
        )}
        style={{
          fontSize: Math.max(10, size * 0.42),
          background: showImage ? 'var(--color-surface)' : `linear-gradient(135deg, ${site.accent}, color-mix(in oklab, ${site.accent} 55%, white))`,
          boxShadow: shadow,
        }}
      >
        {showImage ? (
          <img
            key={candidates[index]}
            src={candidates[index]}
            alt=""
            className="h-[62%] w-[62%] object-contain"
            onError={() => setIndex((i) => i + 1)}
          />
        ) : (
          site.name.slice(0, 1).toUpperCase()
        )}
      </span>
      {showHat && (
        <BirthdayHat
          size={Math.max(12, size * 0.5)}
          rotate={-12}
          className="absolute -top-2 left-1/2 z-10 -translate-x-1/2"
        />
      )}
    </span>
  )
}
