import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { uid } from './format'

export async function openExternalReplyWindow(url: string, commentToCopy: string | undefined, onClosed?: () => void) {
  if (commentToCopy) {
    await writeText(commentToCopy).catch(() => {})
  }
  const win = new WebviewWindow(`reply-${uid()}`, {
    url,
    title: 'Reply on site',
    width: 1150,
    height: 840,
    center: true,
  })
  if (onClosed) {
    win.once('tauri://destroyed', () => onClosed())
  }
  return win
}

export function openAuthWindow(url: string, title = 'Sign in') {
  return new WebviewWindow(`auth-${uid()}`, {
    url,
    title,
    width: 480,
    height: 680,
    center: true,
  })
}
