import { useState } from 'react'
import { motion } from 'motion/react'
import { Eye, Plus } from 'lucide-react'
import { useSitesStore } from '../../store/useSitesStore'
import { useNavStore } from '../../store/useNavStore'
import { useSettingsStore } from '../../store/useSettingsStore'
import { cn } from '../../lib/cn'
import { AddSiteDialog } from './AddSiteDialog'
import { SiteAvatar } from './SiteAvatar'

export function SiteSwitcher() {
  const allSites = useSitesStore((s) => s.sites)
  const loadBoards = useSitesStore((s) => s.loadBoards)
  const currentSiteId = useNavStore((s) => s.siteId)
  const setSite = useNavStore((s) => s.setSite)
  const hideNsfwSites = useSettingsStore((s) => s.hideNsfwSites)
  const [showAdd, setShowAdd] = useState(false)
  const [revealHidden, setRevealHidden] = useState(false)

  const hiddenCount = allSites.filter((s) => s.nsfw && s.id !== currentSiteId).length
  const sites = allSites.filter((s) => !hideNsfwSites || revealHidden || !s.nsfw || s.id === currentSiteId)

  return (
    <div className="flex items-center gap-3 overflow-x-auto pb-1.5 pt-3">
      {sites.map((site) => {
        const active = site.id === currentSiteId
        return (
          <motion.button
            key={site.id}
            type="button"
            onClick={() => {
              setSite(site.id)
              loadBoards(site.id)
            }}
            whileHover={{ y: -2 }}
            whileTap={{ scale: 0.94 }}
            title={site.name}
            className="group relative flex shrink-0 flex-col items-center gap-1.5"
          >
            <SiteAvatar site={site} size={44} ring={active} />
            <span
              className={cn(
                'max-w-14 truncate text-[11px] text-ink-faint transition-colors',
                active && 'text-ink-dim',
              )}
            >
              {site.name}
            </span>
          </motion.button>
        )
      })}
      <motion.button
        type="button"
        whileHover={{ y: -2 }}
        whileTap={{ scale: 0.94 }}
        onClick={() => setShowAdd(true)}
        title="Add a custom imageboard site"
        className="flex shrink-0 flex-col items-center gap-1.5"
      >
        <span className="flex size-11 items-center justify-center rounded-full border border-dashed border-border text-ink-faint transition-colors hover:border-accent hover:text-accent">
          <Plus size={19} />
        </span>
        <span className="text-[11px] text-ink-faint">Add</span>
      </motion.button>

      {hideNsfwSites && !revealHidden && hiddenCount > 0 && (
        <button
          type="button"
          onClick={() => setRevealHidden(true)}
          title="Show hidden 18+ sites"
          className="flex shrink-0 flex-col items-center gap-1.5"
        >
          <span className="flex size-11 items-center justify-center rounded-full border border-dashed border-border text-ink-faint transition-colors hover:border-accent hover:text-accent">
            <Eye size={17} />
          </span>
          <span className="text-[11px] text-ink-faint">+{hiddenCount}</span>
        </button>
      )}

      <AddSiteDialog open={showAdd} onClose={() => setShowAdd(false)} />
    </div>
  )
}
