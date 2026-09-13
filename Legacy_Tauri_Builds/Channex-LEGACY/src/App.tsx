import { useEffect } from 'react'
import { AnimatePresence, motion } from 'motion/react'
import { TitleBar } from './components/layout/TitleBar'
import { Sidebar } from './components/layout/Sidebar'
import { AnimatedBackground } from './components/layout/AnimatedBackground'
import { OnboardingWizard } from './components/onboarding/OnboardingWizard'
import { CatalogGrid } from './components/catalog/CatalogGrid'
import { ThreadView } from './components/thread/ThreadView'
import { BookmarksPanel } from './components/bookmarks/BookmarksPanel'
import { DownloadsPanel } from './components/downloads/DownloadsPanel'
import { SettingsPanel } from './components/settings/SettingsPanel'
import { MediaLightbox } from './components/lightbox/MediaLightbox'
import { EmptyState } from './components/common/EmptyState'
import { useNavStore } from './store/useNavStore'
import { useSitesStore } from './store/useSitesStore'
import { useBookmarksStore } from './store/useBookmarksStore'
import { useSettingsStore } from './store/useSettingsStore'
import { useApplyTheme } from './hooks/useApplyTheme'
import { useKeyboardShortcuts } from './hooks/useKeyboardShortcuts'
import { LayoutGrid } from 'lucide-react'

function MainView() {
  const view = useNavStore((s) => s.view)
  const siteId = useNavStore((s) => s.siteId)
  const boardCode = useNavStore((s) => s.boardCode)
  const threadId = useNavStore((s) => s.threadId)

  let content: React.ReactNode
  let key: string

  if (view === 'bookmarks') {
    content = <BookmarksPanel />
    key = 'bookmarks'
  } else if (view === 'downloads') {
    content = <DownloadsPanel />
    key = 'downloads'
  } else if (view === 'settings') {
    content = <SettingsPanel />
    key = 'settings'
  } else if (view === 'thread' && boardCode && threadId) {
    content = <ThreadView siteId={siteId} boardCode={boardCode} threadId={threadId} />
    key = `thread-${siteId}-${boardCode}-${threadId}`
  } else if (view === 'catalog' && boardCode) {
    content = <CatalogGrid siteId={siteId} boardCode={boardCode} />
    key = `catalog-${siteId}-${boardCode}`
  } else {
    content = (
      <EmptyState icon={LayoutGrid} title="Pick a board" description="Choose a site and a board from the sidebar to start browsing." />
    )
    key = 'empty'
  }

  return (
    <AnimatePresence mode="wait">
      <motion.div
        key={key}
        initial={{ opacity: 0, x: 8 }}
        animate={{ opacity: 1, x: 0 }}
        exit={{ opacity: 0, x: -8 }}
        transition={{ duration: 0.2, ease: [0.16, 1, 0.3, 1] }}
        className="flex min-h-0 flex-1 flex-col"
      >
        {content}
      </motion.div>
    </AnimatePresence>
  )
}

function App() {
  const hydrateSites = useSitesStore((s) => s.hydrate)
  const hydrateBookmarks = useBookmarksStore((s) => s.hydrate)
  const hydrateSettings = useSettingsStore((s) => s.hydrate)
  const settingsHydrated = useSettingsStore((s) => s.hydrated)
  const hasCompletedOnboarding = useSettingsStore((s) => s.hasCompletedOnboarding)

  useEffect(() => {
    hydrateSites()
    hydrateBookmarks()
    hydrateSettings()
  }, [hydrateSites, hydrateBookmarks, hydrateSettings])

  useApplyTheme()
  useKeyboardShortcuts()

  if (!settingsHydrated) {
    return <div className="h-screen w-screen bg-canvas" />
  }

  if (!hasCompletedOnboarding) {
    return (
      <div className="h-screen w-screen overflow-hidden text-ink">
        <AnimatedBackground />
        <OnboardingWizard />
      </div>
    )
  }

  return (
    <div className="flex h-screen w-screen flex-col overflow-hidden text-ink">
      <AnimatedBackground />
      <TitleBar />
      <div className="flex min-h-0 flex-1">
        <Sidebar />
        <main className="flex min-h-0 flex-1 flex-col">
          <MainView />
        </main>
      </div>
      <MediaLightbox />
    </div>
  )
}

export default App
