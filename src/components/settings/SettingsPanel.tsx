import { FolderOpen, RotateCcw, Trash2 } from 'lucide-react'
import { useSettingsStore } from '../../store/useSettingsStore'
import { useSitesStore } from '../../store/useSitesStore'
import { pickDownloadDir } from '../../lib/download'
import { ThemePicker } from './ThemePicker'
import { AccentPicker } from './AccentPicker'
import { BackgroundPicker } from './BackgroundPicker'
import { ViewModeToggle } from '../catalog/ViewModeToggle'

export function SettingsPanel() {
  const downloadDir = useSettingsStore((s) => s.downloadDir)
  const setDownloadDir = useSettingsStore((s) => s.setDownloadDir)
  const blurNsfw = useSettingsStore((s) => s.blurNsfw)
  const setBlurNsfw = useSettingsStore((s) => s.setBlurNsfw)
  const hideNsfwSites = useSettingsStore((s) => s.hideNsfwSites)
  const setHideNsfwSites = useSettingsStore((s) => s.setHideNsfwSites)
  const muteWebmsByDefault = useSettingsStore((s) => s.muteWebmsByDefault)
  const setMuteWebmsByDefault = useSettingsStore((s) => s.setMuteWebmsByDefault)
  const birthdayHats = useSettingsStore((s) => s.birthdayHats)
  const setBirthdayHats = useSettingsStore((s) => s.setBirthdayHats)
  const setHasCompletedOnboarding = useSettingsStore((s) => s.setHasCompletedOnboarding)
  const sites = useSitesStore((s) => s.sites)
  const removeCustomSite = useSitesStore((s) => s.removeCustomSite)

  return (
    <div className="min-h-0 flex-1 overflow-y-auto px-8 py-7">
      <div className="mx-auto max-w-2xl">
        <h1 className="mb-1 text-xl font-semibold text-ink">Settings</h1>
        <p className="mb-8 text-sm text-ink-faint">Tune how Imageboarder looks and behaves.</p>

        <section className="mb-8 space-y-3">
          <p className="section-label">Appearance</p>
          <div className="card space-y-4 p-5">
            <div>
              <p className="mb-2.5 text-sm font-medium text-ink">Theme</p>
              <ThemePicker />
            </div>
            <div className="h-px bg-border-soft" />
            <div>
              <p className="mb-2.5 text-sm font-medium text-ink">Accent color</p>
              <AccentPicker />
            </div>
            <div className="h-px bg-border-soft" />
            <div>
              <p className="mb-2.5 text-sm font-medium text-ink">Animated background</p>
              <BackgroundPicker />
            </div>
          </div>
        </section>

        <section className="mb-8 space-y-3">
          <p className="section-label">Browsing</p>
          <div className="card flex items-center justify-between gap-4 p-4">
            <div>
              <p className="text-sm font-medium text-ink">Thread catalog layout</p>
              <p className="mt-0.5 text-xs text-ink-faint">Switch between a thumbnail grid, a denser grid, or a list view.</p>
            </div>
            <ViewModeToggle />
          </div>
        </section>

        <section className="mb-8 space-y-3">
          <p className="section-label">Downloads</p>
          <div className="card flex items-center gap-3 p-4">
            <div className="min-w-0 flex-1">
              <p className="text-xs text-ink-faint">Default download folder</p>
              <p className="truncate text-sm font-medium text-ink">{downloadDir ?? 'Not set'}</p>
            </div>
            <button
              type="button"
              onClick={async () => {
                const dir = await pickDownloadDir(downloadDir ?? undefined)
                if (dir) await setDownloadDir(dir)
              }}
              className="btn-secondary shrink-0 px-3 py-1.5 text-xs"
            >
              <FolderOpen size={13} /> Choose…
            </button>
          </div>
        </section>

        <section className="mb-8 space-y-3">
          <p className="section-label">Content</p>
          <div className="card divide-y divide-border-soft">
            <label className="flex cursor-pointer items-center justify-between gap-4 p-4">
              <div>
                <p className="text-sm font-medium text-ink">Blur NSFW / spoiler thumbnails</p>
                <p className="mt-0.5 text-xs text-ink-faint">Blur images on 18+ sites and spoilered files until you tap to reveal.</p>
              </div>
              <input type="checkbox" checked={blurNsfw} onChange={(e) => setBlurNsfw(e.target.checked)} className="size-4 shrink-0 accent-accent" />
            </label>
            <label className="flex cursor-pointer items-center justify-between gap-4 p-4">
              <div>
                <p className="text-sm font-medium text-ink">Hide 18+ sites from the site list</p>
                <p className="mt-0.5 text-xs text-ink-faint">Sites marked 18+ (like 8kun and 8chan.moe) are hidden from the switcher until you reveal them.</p>
              </div>
              <input type="checkbox" checked={hideNsfwSites} onChange={(e) => setHideNsfwSites(e.target.checked)} className="size-4 shrink-0 accent-accent" />
            </label>
            <label className="flex cursor-pointer items-center justify-between gap-4 p-4">
              <div>
                <p className="text-sm font-medium text-ink">Play WEBMs muted by default</p>
                <p className="mt-0.5 text-xs text-ink-faint">Video previews autoplay silently until you unmute them. Turn off to autoplay with sound (browsers may then require a manual tap to start playback).</p>
              </div>
              <input
                type="checkbox"
                checked={muteWebmsByDefault}
                onChange={(e) => setMuteWebmsByDefault(e.target.checked)}
                className="size-4 shrink-0 accent-accent"
              />
            </label>
            <label className="flex cursor-pointer items-center justify-between gap-4 p-4">
              <div>
                <p className="text-sm font-medium text-ink">Birthday hats</p>
                <p className="mt-0.5 text-xs text-ink-faint">On 4chan's birthday (October 1st), show a party hat on site logos and every post.</p>
              </div>
              <input
                type="checkbox"
                checked={birthdayHats}
                onChange={(e) => setBirthdayHats(e.target.checked)}
                className="size-4 shrink-0 accent-accent"
              />
            </label>
          </div>
        </section>

        <section className="mb-8 space-y-3">
          <p className="section-label">Custom sites</p>
          {sites.filter((s) => s.isCustom).length === 0 ? (
            <div className="card p-5 text-center text-xs text-ink-faint">No custom sites added. Use the "+" button in the sidebar to add one.</div>
          ) : (
            <div className="card divide-y divide-border-soft">
              {sites
                .filter((s) => s.isCustom)
                .map((s) => (
                  <div key={s.id} className="flex items-center gap-3 px-4 py-3">
                    <span className="size-2.5 shrink-0 rounded-full" style={{ background: s.accent }} />
                    <span className="text-sm font-medium text-ink">{s.name}</span>
                    <span className="truncate text-xs text-ink-faint">{s.siteOrigin}</span>
                    <button type="button" onClick={() => removeCustomSite(s.id)} className="btn-icon ml-auto size-7 hover:text-red-400">
                      <Trash2 size={13} />
                    </button>
                  </div>
                ))}
            </div>
          )}
        </section>

        <section className="space-y-3">
          <p className="section-label">About</p>
          <div className="card flex items-center gap-3 p-4">
            <div className="min-w-0 flex-1">
              <p className="text-sm font-medium text-ink">First-time setup</p>
              <p className="mt-0.5 text-xs text-ink-faint">Replay the welcome guide to reconfigure appearance and content preferences.</p>
            </div>
            <button type="button" onClick={() => setHasCompletedOnboarding(false)} className="btn-secondary shrink-0 px-3 py-1.5 text-xs">
              <RotateCcw size={13} /> Replay setup
            </button>
          </div>
        </section>

        <p className="pt-4 text-center text-xs text-ink-faint">- niruxxdaboi -</p>
      </div>
    </div>
  )
}
