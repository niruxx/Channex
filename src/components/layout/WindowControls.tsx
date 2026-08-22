import { useEffect, useState } from 'react'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { Minus, Square, Copy, X } from 'lucide-react'
import { isMac } from '../../lib/platform'
import { cn } from '../../lib/cn'

const appWindow = getCurrentWindow()

export function WindowControls() {
  const [maximized, setMaximized] = useState(false)
  const mac = isMac()

  useEffect(() => {
    appWindow.isMaximized().then(setMaximized).catch(() => {})
    const unlisten = appWindow.onResized(() => {
      appWindow.isMaximized().then(setMaximized).catch(() => {})
    })
    return () => {
      unlisten.then((fn) => fn())
    }
  }, [])

  if (mac) {
    return (
      <div className="flex shrink-0 items-center gap-2.5 pl-1">
        <TrafficLight color="#ff5f57" onClick={() => appWindow.close()} />
        <TrafficLight color="#febc2e" onClick={() => appWindow.minimize()} />
        <TrafficLight color="#28c840" onClick={() => appWindow.toggleMaximize()} />
      </div>
    )
  }

  return (
    <div className="flex shrink-0 items-center">
      <CtrlButton onClick={() => appWindow.minimize()} title="Minimize">
        <Minus size={16} />
      </CtrlButton>
      <CtrlButton onClick={() => appWindow.toggleMaximize()} title={maximized ? 'Restore' : 'Maximize'}>
        {maximized ? <Copy size={13} /> : <Square size={13} />}
      </CtrlButton>
      <CtrlButton onClick={() => appWindow.close()} title="Close" danger>
        <X size={16} />
      </CtrlButton>
    </div>
  )
}

function CtrlButton({
  children,
  onClick,
  title,
  danger,
}: {
  children: React.ReactNode
  onClick: () => void
  title: string
  danger?: boolean
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      title={title}
      className={cn(
        'flex h-14 w-14 items-center justify-center text-ink-faint transition-colors',
        danger ? 'hover:bg-red-500 hover:text-white' : 'hover:bg-surface-3 hover:text-ink',
      )}
    >
      {children}
    </button>
  )
}

function TrafficLight({ color, onClick }: { color: string; onClick: () => void }) {
  return (
    <button
      type="button"
      onClick={onClick}
      className="group flex size-4 items-center justify-center rounded-full transition-transform active:scale-90"
      style={{ background: color }}
    />
  )
}
