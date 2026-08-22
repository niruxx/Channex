import { useState } from 'react'
import { AnimatePresence, motion } from 'motion/react'
import { ArrowRight, Bookmark, CheckCircle2, Download, FolderOpen, LayoutGrid, MessagesSquare, Reply } from 'lucide-react'
import { useSettingsStore } from '../../store/useSettingsStore'
import { pickDownloadDir } from '../../lib/download'
import { ThemePicker } from '../settings/ThemePicker'
import { AccentPicker } from '../settings/AccentPicker'
import { BackgroundPicker } from '../settings/BackgroundPicker'
import { cn } from '../../lib/cn'

const STEPS = ['welcome', 'appearance', 'content', 'downloads', 'finish'] as const
type Step = (typeof STEPS)[number]

export function OnboardingWizard() {
  const [stepIndex, setStepIndex] = useState(0)
  const step: Step = STEPS[stepIndex]
  const setHasCompletedOnboarding = useSettingsStore((s) => s.setHasCompletedOnboarding)

  const isFirst = stepIndex === 0
  const isLast = stepIndex === STEPS.length - 1

  function next() {
    if (isLast) {
      setHasCompletedOnboarding(true)
      return
    }
    setStepIndex((i) => Math.min(i + 1, STEPS.length - 1))
  }
  function back() {
    setStepIndex((i) => Math.max(i - 1, 0))
  }
  function skip() {
    setHasCompletedOnboarding(true)
  }

  return (
    <div className="relative z-10 flex h-full flex-col items-center justify-center p-6">
      {!isLast && (
        <button
          type="button"
          onClick={skip}
          className="absolute right-6 top-6 rounded-lg px-2.5 py-1.5 text-xs font-medium text-ink-faint transition-colors hover:bg-surface-3 hover:text-ink"
        >
          Skip setup
        </button>
      )}

      <div className="w-full max-w-xl">
        <div className="mb-7 flex items-center justify-center gap-1.5">
          {STEPS.map((s, i) => (
            <span
              key={s}
              className={cn(
                'h-1.5 rounded-full transition-all duration-300',
                i === stepIndex ? 'w-7 bg-accent' : i < stepIndex ? 'w-1.5 bg-accent/50' : 'w-1.5 bg-surface-4',
              )}
            />
          ))}
        </div>

        <div className="card overflow-hidden p-9" style={{ boxShadow: 'var(--shadow-lg)' }}>
          <AnimatePresence mode="wait" initial={false}>
            <motion.div
              key={step}
              initial={{ opacity: 0, x: 16 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0, x: -16 }}
              transition={{ duration: 0.25, ease: [0.16, 1, 0.3, 1] }}
            >
              {step === 'welcome' && <WelcomeStep />}
              {step === 'appearance' && <AppearanceStep />}
              {step === 'content' && <ContentStep />}
              {step === 'downloads' && <DownloadsStep />}
              {step === 'finish' && <FinishStep />}
            </motion.div>
          </AnimatePresence>

          <div className="mt-8 flex items-center justify-between">
            {isFirst ? (
              <span />
            ) : (
              <button type="button" onClick={back} className="btn-ghost">
                Back
              </button>
            )}
            <button type="button" onClick={next} className="btn-primary px-5">
              {isLast ? 'Start browsing' : step === 'welcome' ? 'Get started' : 'Continue'}
              {!isLast && <ArrowRight size={14} />}
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}

function WelcomeStep() {
  return (
    <div className="flex flex-col items-center gap-4 text-center">
      <div
        className="animate-float flex size-20 items-center justify-center rounded-3xl text-accent-ink"
        style={{ background: 'linear-gradient(135deg, var(--color-accent), var(--color-accent-2))', boxShadow: 'var(--shadow-md)' }}
      >
        <MessagesSquare size={34} strokeWidth={1.75} />
      </div>
      <div className="space-y-1.5">
        <h1 className="text-xl font-semibold text-ink">Welcome to Imageboarder</h1>
        <p className="text-sm leading-relaxed text-ink-dim">
          A fast, modern viewer for 4chan, 8kun, and other imageboards — browse, bookmark, download, and reply, all in
          one app.
        </p>
      </div>
      <div className="grid w-full grid-cols-3 gap-2.5 pt-2 text-xs text-ink-faint">
        <Feature icon={LayoutGrid} label="Browse & switch sites" />
        <Feature icon={Bookmark} label="Bookmark threads" />
        <Feature icon={Reply} label="Reply & download" />
      </div>
    </div>
  )
}

function Feature({ icon: Icon, label }: { icon: typeof LayoutGrid; label: string }) {
  return (
    <div className="flex flex-col items-center gap-1.5 rounded-xl bg-surface-3 px-2.5 py-3.5">
      <Icon size={17} className="text-accent" />
      <span className="text-center leading-tight">{label}</span>
    </div>
  )
}

function AppearanceStep() {
  return (
    <div className="space-y-5">
      <div className="text-center">
        <h2 className="text-lg font-semibold text-ink">Make it yours</h2>
        <p className="mt-1 text-sm text-ink-faint">Pick a theme, an accent color, and an animated background.</p>
      </div>
      <div>
        <p className="mb-2.5 text-sm font-medium text-ink">Theme</p>
        <ThemePicker />
      </div>
      <div>
        <p className="mb-2.5 text-sm font-medium text-ink">Accent color</p>
        <AccentPicker />
      </div>
      <div>
        <p className="mb-2.5 text-sm font-medium text-ink">Animated background</p>
        <BackgroundPicker />
      </div>
    </div>
  )
}

function ContentStep() {
  const blurNsfw = useSettingsStore((s) => s.blurNsfw)
  const setBlurNsfw = useSettingsStore((s) => s.setBlurNsfw)
  const hideNsfwSites = useSettingsStore((s) => s.hideNsfwSites)
  const setHideNsfwSites = useSettingsStore((s) => s.setHideNsfwSites)

  return (
    <div className="space-y-5">
      <div className="text-center">
        <h2 className="text-lg font-semibold text-ink">Content preferences</h2>
        <p className="mt-1 text-sm text-ink-faint">Some imageboards host 18+ content. You can change these anytime.</p>
      </div>
      <div className="space-y-2.5">
        <label className="flex cursor-pointer items-center justify-between gap-4 rounded-xl border border-border-soft bg-surface-3 p-4">
          <div>
            <p className="text-sm font-medium text-ink">Blur NSFW / spoiler thumbnails</p>
            <p className="mt-0.5 text-xs text-ink-faint">Blur images on 18+ sites and spoilered files until you tap to reveal.</p>
          </div>
          <input type="checkbox" checked={blurNsfw} onChange={(e) => setBlurNsfw(e.target.checked)} className="size-5 shrink-0 accent-accent" />
        </label>
        <label className="flex cursor-pointer items-center justify-between gap-4 rounded-xl border border-border-soft bg-surface-3 p-4">
          <div>
            <p className="text-sm font-medium text-ink">Hide 18+ sites from the site list</p>
            <p className="mt-0.5 text-xs text-ink-faint">Sites like 8kun and 8chan.moe stay out of the switcher until revealed.</p>
          </div>
          <input type="checkbox" checked={hideNsfwSites} onChange={(e) => setHideNsfwSites(e.target.checked)} className="size-5 shrink-0 accent-accent" />
        </label>
      </div>
    </div>
  )
}

function DownloadsStep() {
  const downloadDir = useSettingsStore((s) => s.downloadDir)
  const setDownloadDir = useSettingsStore((s) => s.setDownloadDir)

  return (
    <div className="space-y-5">
      <div className="text-center">
        <h2 className="text-lg font-semibold text-ink">Where should downloads go?</h2>
        <p className="mt-1 text-sm text-ink-faint">Pick a default folder for saved images and threads. Optional — you can skip this.</p>
      </div>
      <div className="flex items-center gap-3 rounded-xl border border-border-soft bg-surface-3 p-4">
        <Download size={18} className="shrink-0 text-accent" />
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
          className="btn-secondary shrink-0 px-3.5 py-2 text-sm"
        >
          <FolderOpen size={14} /> Choose…
        </button>
      </div>
    </div>
  )
}

function FinishStep() {
  return (
    <div className="flex flex-col items-center gap-4 py-2 text-center">
      <div
        className="flex size-16 items-center justify-center rounded-full text-accent-ink"
        style={{ background: 'linear-gradient(135deg, var(--color-accent), var(--color-accent-2))', boxShadow: 'var(--shadow-md)' }}
      >
        <CheckCircle2 size={30} />
      </div>
      <div className="space-y-1.5">
        <h2 className="text-lg font-semibold text-ink">You're all set</h2>
        <p className="text-sm leading-relaxed text-ink-dim">
          Everything's configured — you can fine-tune any of it later from Settings. Time to start browsing.
        </p>
      </div>
    </div>
  )
}
